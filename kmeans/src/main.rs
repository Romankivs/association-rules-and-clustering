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

    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    fn add(&self, other: &Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn scale(&self, factor: f64) -> Point {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

struct KMeans {
    k: usize,
    max_iterations: usize,
    centroids: Vec<Point>,
}

impl KMeans {
    fn new(k: usize, max_iterations: usize) -> Self {
        KMeans {
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
                    let dist = point.distance(centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        cluster = i;
                    }
                }
                
                cluster
            })
            .collect()
    }
    
    fn update_centroids(&mut self, data: &[Point], clusters: &[usize]) -> bool {
        let mut new_centroids = vec![Point::new(0.0, 0.0); self.k];
        let mut counts = vec![0; self.k];
        
        for (point, &cluster) in data.iter().zip(clusters.iter()) {
            new_centroids[cluster] = new_centroids[cluster].add(point);
            counts[cluster] += 1;
        }
        
        for i in 0..self.k {
            if counts[i] > 0 {
                new_centroids[i] = new_centroids[i].scale(1.0 / counts[i] as f64);
            } else {
                new_centroids[i] = self.centroids[i];
            }
        }
        
        let mut changed = false;
        for i in 0..self.k {
            if new_centroids[i].distance(&self.centroids[i]) > 1e-6 {
                changed = true;
            }
            self.centroids[i] = new_centroids[i];
        }
        
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
            .map(|(point, &cluster)| point.distance(&self.centroids[cluster]).powi(2))
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
    
    for k in 2..=5 {
        println!("\nRunning k-means with k = {}", k);
        let mut kmeans = KMeans::new(k, 100);
        let clusters = kmeans.fit(&data);
        
        let inertia = kmeans.inertia(&data, &clusters);
        println!("Inertia (sum of squared distances): {:.4}", inertia);
        
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
