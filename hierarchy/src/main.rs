#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone)]
struct Cluster {
    id: usize,
    points: Vec<usize>,
    left: Option<Box<Cluster>>,
    right: Option<Box<Cluster>>,
    height: f64,
}

impl Cluster {
    fn new(id: usize, points: Vec<usize>) -> Self {
        Cluster {
            id,
            points,
            left: None,
            right: None,
            height: 0.0,
        }
    }

    fn merge(id: usize, left: Cluster, right: Cluster, height: f64) -> Self {
        let mut points = left.points.clone();
        points.extend(right.points.clone());
        
        Cluster {
            id,
            points,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            height,
        }
    }
}

enum LinkageMethod {
    Single,
    Complete,
    Average,
}

struct HierarchicalClustering {
    data: Vec<Point>,
    method: LinkageMethod,
}

impl HierarchicalClustering {
    fn new(data: Vec<Point>, method: LinkageMethod) -> Self {
        HierarchicalClustering { data, method }
    }

    fn cluster_distance(&self, cluster_a: &Cluster, cluster_b: &Cluster) -> f64 {
        match self.method {
            LinkageMethod::Single => self.single_link_distance(cluster_a, cluster_b),
            LinkageMethod::Complete => self.complete_link_distance(cluster_a, cluster_b),
            LinkageMethod::Average => self.average_link_distance(cluster_a, cluster_b),
        }
    }

    fn single_link_distance(&self, cluster_a: &Cluster, cluster_b: &Cluster) -> f64 {
        let mut min_distance = f64::INFINITY;
        
        for &point_idx_a in &cluster_a.points {
            for &point_idx_b in &cluster_b.points {
                let distance = self.data[point_idx_a].distance(&self.data[point_idx_b]);
                if distance < min_distance {
                    min_distance = distance;
                }
            }
        }
        
        min_distance
    }

    fn complete_link_distance(&self, cluster_a: &Cluster, cluster_b: &Cluster) -> f64 {
        let mut max_distance = 0.0;
        
        for &point_idx_a in &cluster_a.points {
            for &point_idx_b in &cluster_b.points {
                let distance = self.data[point_idx_a].distance(&self.data[point_idx_b]);
                if distance > max_distance {
                    max_distance = distance;
                }
            }
        }
        
        max_distance
    }

    fn average_link_distance(&self, cluster_a: &Cluster, cluster_b: &Cluster) -> f64 {
        let mut sum_distance = 0.0;
        let mut count = 0;
        
        for &point_idx_a in &cluster_a.points {
            for &point_idx_b in &cluster_b.points {
                sum_distance += self.data[point_idx_a].distance(&self.data[point_idx_b]);
                count += 1;
            }
        }
        
        if count > 0 {
            sum_distance / count as f64
        } else {
            f64::INFINITY
        }
    }

    fn find_closest_clusters(&self, clusters: &[Cluster]) -> (usize, usize, f64) {
        let mut min_distance = f64::INFINITY;
        let mut closest_pair = (0, 1);
        
        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let distance = self.cluster_distance(&clusters[i], &clusters[j]);
                if distance < min_distance {
                    min_distance = distance;
                    closest_pair = (i, j);
                }
            }
        }
        
        (closest_pair.0, closest_pair.1, min_distance)
    }

    fn fit(&self) -> Cluster {
        let mut clusters: Vec<Cluster> = self.data.iter().enumerate()
            .map(|(i, _)| Cluster::new(i, vec![i]))
            .collect();
        
        let mut next_cluster_id = self.data.len();
        
        while clusters.len() > 1 {
            let (i, j, distance) = self.find_closest_clusters(&clusters);
            
            let cluster_i = clusters.remove(if i > j { i } else { i });
            let cluster_j = clusters.remove(if i > j { j } else { j - 1 });
            
            let merged_cluster = Cluster::merge(
                next_cluster_id,
                cluster_i,
                cluster_j,
                distance
            );
            
            next_cluster_id += 1;
            clusters.push(merged_cluster);
        }
        
        clusters.pop().unwrap()
    }

    fn print_dendrogram(&self, node: &Cluster, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}Cluster {} (height: {:.2})", indent, node.id, node.height);
        
        if node.points.len() <= 3 {
            println!("{}Points: {:?}", indent, node.points.iter()
                .map(|&idx| format!("({:.1},{:.1})", self.data[idx].x, self.data[idx].y))
                .collect::<Vec<_>>());
        } else {
            println!("{}Contains {} points", indent, node.points.len());
        }
        
        if let Some(ref left) = node.left {
            self.print_dendrogram(left, depth + 1);
        }
        if let Some(ref right) = node.right {
            self.print_dendrogram(right, depth + 1);
        }
    }
}

fn run_clustering(data: &[Point], method: LinkageMethod, method_name: &str) {
    println!("\n=== {} Linkage Hierarchical Clustering ===", method_name);
    
    let clustering = HierarchicalClustering::new(data.to_vec(), method);
    let dendrogram = clustering.fit();
    
    println!("\nDendrogram structure:");
    clustering.print_dendrogram(&dendrogram, 0);
}

fn main() {
    let data = vec![
        Point::new(1.0, 1.0),
        Point::new(1.0, 8.0),
        Point::new(2.0, 2.0),
        Point::new(2.0, 5.0),
        Point::new(3.0, 1.0),
        Point::new(4.0, 3.0),
        Point::new(5.0, 2.0),
        Point::new(6.0, 1.0),
        Point::new(6.0, 8.0),
        Point::new(8.0, 6.0),
    ];
    
    run_clustering(&data, LinkageMethod::Single, "Single");
    run_clustering(&data, LinkageMethod::Complete, "Complete");
    run_clustering(&data, LinkageMethod::Average, "Average");
}
