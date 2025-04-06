use std::collections::{HashMap, HashSet};

type ItemSet = Vec<char>;
type Transaction = HashSet<char>;
type SupportCount = usize;

fn generate_candidates(l_prev: &[ItemSet], k: usize) -> Vec<ItemSet> {
    let mut result = Vec::new();

    for (i, p) in l_prev.iter().enumerate() {
        let p_set: HashSet<char> = p.iter().cloned().collect();

        for q in l_prev.iter().skip(i + 1) {
            let q_set: HashSet<char> = q.iter().cloned().collect();

            if p.len() >= k - 1 && q.len() >= k - 1 {
                let mut can_join = true;
                for i in 0..k - 2 {
                    if p[i] != q[i] {
                        can_join = false;
                        break;
                    }
                }

                if can_join && p[k - 2] != q[k - 2] {
                    let mut union = p_set.union(&q_set).cloned().collect::<Vec<char>>();
                    union.sort();

                    if union.len() == k {
                        let mut is_valid = true;

                        for i in 0..k {
                            let mut subset = union.clone();
                            subset.remove(i);

                            let mut found = false;
                            for existing in l_prev {
                                if subset == *existing {
                                    found = true;
                                    break;
                                }
                            }

                            if !found {
                                is_valid = false;
                                break;
                            }
                        }

                        if is_valid {
                            result.push(union);
                        }
                    }
                }
            }
        }
    }

    result
}

fn calculate_support(
    candidates: &[ItemSet],
    transactions: &[Transaction],
) -> HashMap<Vec<char>, SupportCount> {
    let mut counts = HashMap::new();

    for c in candidates {
        let c_set: HashSet<_> = c.iter().cloned().collect();
        let mut count = 0;

        for t in transactions {
            if c_set.is_subset(t) {
                count += 1;
            }
        }

        counts.insert(c.clone(), count);
    }

    counts
}

fn get_frequent_itemsets(
    candidates: &[ItemSet],
    support_counts: &HashMap<Vec<char>, SupportCount>,
    min_support: f64,
    transaction_count: usize,
) -> Vec<ItemSet> {
    let min_count = (min_support * transaction_count as f64).ceil() as usize;

    candidates
        .iter()
        .filter(|c| support_counts.get(*c).unwrap_or(&0) >= &min_count)
        .cloned()
        .collect()
}

fn generate_all_subsets(itemset: &[char]) -> Vec<Vec<char>> {
    let n = itemset.len();
    let mut result = Vec::new();

    for mask in 1..(1 << n) - 1 {
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

fn generate_rules(
    frequent_itemsets: &[ItemSet],
    support_counts: &HashMap<Vec<char>, SupportCount>,
    min_confidence: f64,
) -> Vec<(Vec<char>, Vec<char>, f64)> {
    let mut rules = Vec::new();

    for itemset in frequent_itemsets {
        if itemset.len() <= 1 {
            continue;
        }

        let itemset_support = *support_counts.get(itemset).unwrap_or(&0) as f64;

        let subsets = generate_all_subsets(itemset);

        for antecedent in &subsets {
            let antecedent_set: HashSet<_> = antecedent.iter().cloned().collect();
            let itemset_set: HashSet<_> = itemset.iter().cloned().collect();

            let consequent_set: HashSet<_> =
                itemset_set.difference(&antecedent_set).cloned().collect();
            let mut consequent = Vec::from_iter(consequent_set);
            consequent.sort();

            if consequent.is_empty() {
                continue;
            }

            let antecedent_support = *support_counts.get(antecedent).unwrap_or(&0) as f64;
            if antecedent_support == 0.0 {
                continue;
            }

            let confidence = itemset_support / antecedent_support;

            if confidence >= min_confidence {
                rules.push((antecedent.clone(), consequent, confidence));
            }
        }
    }

    rules
}

fn apriori(
    transactions: &[Transaction],
    min_support: f64,
    min_confidence: f64,
) -> (
    Vec<ItemSet>,
    HashMap<Vec<char>, SupportCount>,
    Vec<(Vec<char>, Vec<char>, f64)>,
) {
    let transaction_count = transactions.len();

    let mut unique_items = HashSet::new();
    for t in transactions {
        for item in t {
            unique_items.insert(*item);
        }
    }

    let mut singleton_candidates: Vec<ItemSet> =
        unique_items.iter().map(|item| vec![*item]).collect();

    singleton_candidates.sort();

    let mut all_support_counts = calculate_support(&singleton_candidates, transactions);

    let mut l_prev = get_frequent_itemsets(
        &singleton_candidates,
        &all_support_counts,
        min_support,
        transaction_count,
    );

    let mut k = 2;
    let mut all_frequent_itemsets = l_prev.clone();

    while !l_prev.is_empty() {
        let candidates = generate_candidates(&l_prev, k);

        if candidates.is_empty() {
            break;
        }

        let support_counts = calculate_support(&candidates, transactions);

        let l_k =
            get_frequent_itemsets(&candidates, &support_counts, min_support, transaction_count);

        all_frequent_itemsets.extend(l_k.clone());

        for (key, value) in support_counts {
            all_support_counts.insert(key, value);
        }

        l_prev = l_k;
        k += 1;
    }

    let rules = generate_rules(&all_frequent_itemsets, &all_support_counts, min_confidence);

    (all_frequent_itemsets, all_support_counts, rules)
}

fn main() {
    let transactions: Vec<HashSet<char>> = vec![
        ['a', 'b', 'c', 'd'].iter().cloned().collect(),
        ['b', 'c', 'd'].iter().cloned().collect(),
        ['a', 'e', 'f', 'g', 'h'].iter().cloned().collect(),
        ['b', 'c', 'd', 'e', 'g', 'j'].iter().cloned().collect(),
        ['b', 'c', 'd', 'e', 'f'].iter().cloned().collect(),
        ['a', 'f', 'g'].iter().cloned().collect(),
        ['a', 'i', 'j'].iter().cloned().collect(),
        ['a', 'b', 'e', 'h'].iter().cloned().collect(),
        ['f', 'g', 'h', 'i', 'j'].iter().cloned().collect(),
        ['e', 'f', 'h'].iter().cloned().collect(),
    ];

    let (frequent_itemsets, support_counts, rules) = apriori(&transactions, 0.4, 0.75);

    println!("Frequent Itemsets (with support):");
    for (i, itemset) in frequent_itemsets.iter().enumerate() {
        let support = support_counts.get(itemset).unwrap_or(&0);
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
