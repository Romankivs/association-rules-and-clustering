use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt;

type Transaction = Vec<char>;
type Support = usize;
type ItemSupport = HashMap<char, Support>;
type FrequentItemsets = Vec<(Vec<char>, Support)>;

struct FPNode {
    item: Option<char>,
    count: usize,
    parent: Option<Rc<RefCell<FPNode>>>,
    children: HashMap<char, Rc<RefCell<FPNode>>>,
    node_link: Option<Rc<RefCell<FPNode>>>,
}

impl FPNode {
    fn new(item: Option<char>, parent: Option<Rc<RefCell<FPNode>>>) -> Self {
        FPNode {
            item,
            count: 0,
            parent,
            children: HashMap::new(),
            node_link: None,
        }
    }

    fn increment(&mut self, count: usize) {
        self.count += count;
    }
}

impl Clone for FPNode {
    fn clone(&self) -> Self {
        FPNode {
            item: self.item.clone(),
            count: self.count,
            parent: self.parent.clone(),
            children: self.children.clone(),
            node_link: self.node_link.clone(),
        }
    }
}

struct HeaderTableEntry {
    support: Support,
    head: Option<Rc<RefCell<FPNode>>>,
}

impl HeaderTableEntry {
    fn new(support: Support) -> Self {
        HeaderTableEntry {
            support,
            head: None,
        }
    }
}

struct FPTree {
    root: Rc<RefCell<FPNode>>,
    header_table: HashMap<char, HeaderTableEntry>,
}

impl fmt::Display for FPTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FP-Tree Structure:")?;
        self.visualize_node(f, &self.root, 0)?;
        
        writeln!(f, "\nHeader Table:")?;
        let mut items: Vec<char> = self.header_table.keys().cloned().collect();
        items.sort();
        
        for item in items {
            let entry = &self.header_table[&item];
            write!(f, "Item {}: Support={}", item, entry.support)?;
            
            if let Some(node) = &entry.head {
                write!(f, ", Links: ")?;
                let mut current = Some(Rc::clone(node));
                while let Some(node_ref) = current {
                    let node_borrow = node_ref.borrow();
                    write!(f, "{}:{} ", node_borrow.item.unwrap(), node_borrow.count)?;
                    current = node_borrow.node_link.clone();
                    if current.is_some() {
                        write!(f, "-> ")?;
                    }
                }
            }
            writeln!(f)?;
        }
        
        Ok(())
    }
}


impl FPTree {
    fn new() -> Self {
        FPTree {
            root: Rc::new(RefCell::new(FPNode::new(None, None))),
            header_table: HashMap::new(),
        }
    }

    fn visualize_node(&self, f: &mut fmt::Formatter, node: &Rc<RefCell<FPNode>>, depth: usize) -> fmt::Result {
        let node_borrow = node.borrow();
        let indent = "    ".repeat(depth);
        
        if let Some(item) = node_borrow.item {
            writeln!(f, "{}Item: {}, Count: {}", indent, item, node_borrow.count)?;
        } else {
            writeln!(f, "{}Root", indent)?;
        }
        
        let mut children: Vec<char> = node_borrow.children.keys().cloned().collect();
        children.sort();
        
        for item in children {
            if let Some(child) = node_borrow.children.get(&item) {
                self.visualize_node(f, child, depth + 1)?;
            }
        }
        
        Ok(())
    }

    fn add_transaction(&mut self, transaction: Transaction, count: usize) {
        let mut current_node = Rc::clone(&self.root);

        for item in transaction {
            let child: Rc<RefCell<FPNode>> = {
                let mut node = current_node.borrow_mut();
                if !node.children.contains_key(&item) {
                    let new_child = Rc::new(RefCell::new(FPNode::new(
                        Some(item),
                        Some(Rc::clone(&current_node)),
                    )));
                    node.children.insert(item, Rc::clone(&new_child));

                    let header_entry = self.header_table.get_mut(&item).unwrap();
                    if header_entry.head.is_none() {
                        header_entry.head = Some(Rc::clone(&new_child));
                    } else {
                        let mut current = Rc::clone(header_entry.head.as_ref().unwrap());
                        while current.borrow().node_link.is_some() {
                            let next = Rc::clone(current.borrow().node_link.as_ref().unwrap());
                            current = next;
                        }
                        current.borrow_mut().node_link = Some(Rc::clone(&new_child));
                    }

                    Rc::clone(&new_child)
                } else {
                    Rc::clone(node.children.get(&item).unwrap())
                }
            };

            child.borrow_mut().increment(count);
            current_node = child;
        }
    }

    fn build(&mut self, transactions: &[Transaction], min_support: usize) {
        let mut item_counts: ItemSupport = HashMap::new();
        for transaction in transactions {
            for item in transaction {
                *item_counts.entry(*item).or_insert(0) += 1;
            }
        }

        self.header_table.clear();
        for (item, count) in item_counts.iter() {
            if *count >= min_support {
                self.header_table
                    .insert(*item, HeaderTableEntry::new(*count));
            }
        }

        for transaction in transactions {
            let mut filtered_transaction: Vec<(char, usize)> = transaction
                .iter()
                .filter_map(|item| {
                    if let Some(entry) = self.header_table.get(item) {
                        Some((*item, entry.support))
                    } else {
                        None
                    }
                })
                .collect();

            filtered_transaction.sort_by(|a, b| b.1.cmp(&a.1));

            if !filtered_transaction.is_empty() {
                let sorted_items: Transaction = filtered_transaction
                    .into_iter()
                    .map(|(item, _)| item)
                    .collect();

                self.add_transaction(sorted_items, 1);
            }
        }
    }

    fn mine(&self, min_support: usize) -> FrequentItemsets {
        let mut result = Vec::new();
        self.fp_growth(Vec::new(), min_support, &mut result);
        result
    }

    fn fp_growth(&self, prefix: Vec<char>, min_support: usize, result: &mut FrequentItemsets) {
        let mut sorted_items: Vec<(char, Support)> = self
            .header_table
            .iter()
            .map(|(item, entry)| (*item, entry.support))
            .collect();
        sorted_items.sort_by(|a, b| a.1.cmp(&b.1));

        for (item, support) in sorted_items {
            let mut new_prefix = prefix.clone();
            new_prefix.push(item);

            result.push((new_prefix.clone(), support));

            let mut conditional_pattern_base = Vec::new();
            let mut current_node_option = self.header_table.get(&item).unwrap().head.clone();

            while let Some(current_node) = current_node_option {
                let node = current_node.borrow();
                let count = node.count;

                let mut path = Vec::new();
                let mut parent_option = node.parent.clone();

                while let Some(parent) = parent_option {
                    let parent_node = parent.borrow();
                    if parent_node.item.is_some() {
                        path.push(parent_node.item.unwrap());
                    }
                    parent_option = parent_node.parent.clone();
                }

                path.reverse();

                if !path.is_empty() {
                    conditional_pattern_base.push((path, count));
                }

                current_node_option = node.node_link.clone();
            }

            if !conditional_pattern_base.is_empty() {
                let mut conditional_tree = FPTree::new();
                let mut conditional_item_counts: ItemSupport = HashMap::new();

                for (path, count) in &conditional_pattern_base {
                    for item in path {
                        *conditional_item_counts.entry(*item).or_insert(0) += count;
                    }
                }

                for (item, count) in conditional_item_counts.iter() {
                    if *count >= min_support {
                        conditional_tree
                            .header_table
                            .insert(*item, HeaderTableEntry::new(*count));
                    }
                }

                for (path, count) in conditional_pattern_base {
                    let mut filtered_path: Vec<(char, usize)> = path
                        .iter()
                        .filter_map(|item| {
                            if let Some(entry) = conditional_tree.header_table.get(item) {
                                Some((*item, entry.support))
                            } else {
                                None
                            }
                        })
                        .collect();

                    filtered_path.sort_by(|a, b| b.1.cmp(&a.1));

                    if !filtered_path.is_empty() {
                        let sorted_items: Transaction =
                            filtered_path.into_iter().map(|(item, _)| item).collect();

                        conditional_tree.add_transaction(sorted_items, count);
                    }
                }

                if !conditional_tree.header_table.is_empty() {
                    conditional_tree.fp_growth(new_prefix, min_support, result);
                }
            }
        }
    }
}

fn generate_rules(
    frequent_itemsets: &FrequentItemsets,
    min_confidence: f64,
) -> Vec<(Vec<char>, Vec<char>, f64)> {
    let mut rules = Vec::new();

    let mut support_map: HashMap<Vec<char>, Support> = HashMap::new();
    for (itemset, support) in frequent_itemsets {
        support_map.insert(itemset.clone(), *support);
    }

    for (itemset, support) in frequent_itemsets {
        if itemset.len() <= 1 {
            continue;
        }

        let subsets = generate_all_subsets(itemset);

        for subset in &subsets {
            if subset.is_empty() || subset.len() == itemset.len() {
                continue;
            }

            let consequent: Vec<char> = itemset
                .iter()
                .filter(|item| !subset.contains(item))
                .cloned()
                .collect();

            if consequent.is_empty() {
                continue;
            }

            let subset_support = *support_map.get(subset).unwrap_or(&0);
            if subset_support == 0 {
                continue;
            }

            let confidence = *support as f64 / subset_support as f64;

            if confidence >= min_confidence {
                rules.push((subset.clone(), consequent, confidence));
            }
        }
    }

    rules
}

fn generate_all_subsets(itemset: &[char]) -> Vec<Vec<char>> {
    let n = itemset.len();
    let mut result = Vec::new();

    for mask in 0..(1 << n) {
        let mut subset = Vec::new();
        for i in 0..n {
            if (mask >> i) & 1 == 1 {
                subset.push(itemset[i]);
            }
        }
        result.push(subset);
    }

    result
}

fn fp_growth(
    transactions: &[Vec<char>],
    min_support: f64,
    min_confidence: f64,
) -> (FrequentItemsets, Vec<(Vec<char>, Vec<char>, f64)>) {
    let min_support = (min_support * transactions.len() as f64).ceil() as usize;

    let mut fp_tree = FPTree::new();
    fp_tree.build(transactions, min_support);

    println!("{}", fp_tree);

    let frequent_itemsets = fp_tree.mine(min_support);

    let rules = generate_rules(&frequent_itemsets, min_confidence);

    (frequent_itemsets, rules)
}

fn main() {
    let transactions: Vec<Vec<char>> = vec![
        vec!['a', 'b', 'c', 'd'],
        vec!['b', 'c', 'd'],
        vec!['a', 'e', 'f', 'g', 'h'],
        vec!['b', 'c', 'd', 'e', 'g', 'j'],
        vec!['b', 'c', 'd', 'e', 'f'],
        vec!['a', 'f', 'g'],
        vec!['a', 'i', 'j'],
        vec!['a', 'b', 'e', 'h'],
        vec!['f', 'g', 'h', 'i', 'j'],
        vec!['e', 'f', 'h'],
    ];

    let (frequent_itemsets, rules) = fp_growth(&transactions, 0.4, 0.75);

    println!("Frequent Itemsets (with support):");
    for (i, (itemset, support)) in frequent_itemsets.iter().enumerate() {
        let support_percentage = (*support as f64 / transactions.len() as f64) * 100.0;
        println!(
            "{}. {:?} (support: {}/{} = {:.1}%)",
            i + 1,
            itemset,
            support,
            transactions.len(),
            support_percentage
        );
    }

    println!("\nAssociation Rules:");
    for (i, (antecedent, consequent, confidence)) in rules.iter().enumerate() {
        println!(
            "{}. {:?} => {:?} (confidence: {:.2}%)",
            i + 1,
            antecedent,
            consequent,
            confidence * 100.0
        );
    }
}
