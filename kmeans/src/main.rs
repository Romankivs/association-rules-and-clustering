use std::f64;
use std::collections::HashMap;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn manhattan_distance(&self, other: &Point) -> f64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

struct KMedians {
    k: usize,
    max_iterations: usize,
    centroids: Vec<Point>,
}

impl KMedians {
    fn new(k: usize, max_iterations: usize) -> Self {
        KMedians {
            k,
            max_iterations,
            centroids: Vec::new(),
        }
    }

    fn initialize_centroids(&mut self, data: &[Point]) {
        let mut rng = rand::thread_rng();
        let mut centroids = Vec::with_capacity(self.k);
        
        for _ in 0..self.k {
            let index = rng.gen_range(0..data.len());
            centroids.push(data[index]);
        }
        
        self.centroids = centroids;
    }
    
    fn assign_clusters(&self, data: &[Point]) -> Vec<usize> {
        data.iter()
            .map(|point| {
                let mut min_dist = f64::MAX;
                let mut cluster = 0;
                
                for (i, centroid) in self.centroids.iter().enumerate() {
                    let dist = point.manhattan_distance(centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        cluster = i;
                    }
                }
                
                cluster
            })
            .collect()
    }
    
    fn median(values: Vec<f64>) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mut sorted_values = values;
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mid = sorted_values.len() / 2;
        if sorted_values.len() % 2 == 0 {
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[mid]
        }
    }
    
    fn update_centroids(&mut self, data: &[Point], clusters: &[usize]) -> bool {
        let mut new_centroids = vec![Point::new(0.0, 0.0); self.k];
        let mut changed = false;
        
        for i in 0..self.k {
            let mut x_values = Vec::new();
            let mut y_values = Vec::new();
            
            for (point_idx, &cluster) in clusters.iter().enumerate() {
                if cluster == i {
                    x_values.push(data[point_idx].x);
                    y_values.push(data[point_idx].y);
                }
            }
            
            if !x_values.is_empty() {
                let median_x = Self::median(x_values);
                let median_y = Self::median(y_values);
                new_centroids[i] = Point::new(median_x, median_y);
            } else {
                new_centroids[i] = self.centroids[i];
            }
            
            if new_centroids[i].manhattan_distance(&self.centroids[i]) > 1e-6 {
                changed = true;
            }
        }
        
        self.centroids = new_centroids;
        changed
    }
    
    fn fit(&mut self, data: &[Point]) -> Vec<usize> {
        self.initialize_centroids(data);
        
        let mut clusters = self.assign_clusters(data);
        let mut iteration = 0;
        
        while iteration < self.max_iterations {
            let changed = self.update_centroids(data, &clusters);
            
            if !changed {
                break;
            }
            
            clusters = self.assign_clusters(data);
            iteration += 1;
        }
        
        println!("Converged after {} iterations", iteration);
        println!("Final centroids: {:?}", self.centroids);
        
        clusters
    }
    
    fn inertia(&self, data: &[Point], clusters: &[usize]) -> f64 {
        data.iter()
            .zip(clusters.iter())
            .map(|(point, &cluster)| point.manhattan_distance(&self.centroids[cluster]))
            .sum()
    }
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
    
    println!("Running K-Medians Clustering");
    
    for k in 2..=5 {
        println!("\nRunning k-medians with k = {}", k);
        let mut kmedians = KMedians::new(k, 100);
        let clusters = kmedians.fit(&data);
        
        let inertia = kmedians.inertia(&data, &clusters);
        println!("Sum of Manhattan distances: {:.4}", inertia);
        
        println!("Cluster assignments:");
        let mut cluster_map: HashMap<usize, Vec<(f64, f64)>> = HashMap::new();
        
        for (i, point) in data.iter().enumerate() {
            let cluster = clusters[i];
            cluster_map.entry(cluster).or_insert_with(Vec::new).push((point.x, point.y));
        }
        
        for (cluster, points) in cluster_map.iter() {
            println!("Cluster {}: {:?}", cluster, points);
        }
    }
}
