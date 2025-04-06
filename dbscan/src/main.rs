use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq)]
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

struct DBSCAN {
    eps: f64,
    min_points: usize, 
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PointType {
    Core,      
    Border,    
    Noise,     
    Unclassified, 
}

impl DBSCAN {
    fn new(eps: f64, min_points: usize) -> Self {
        DBSCAN { eps, min_points }
    }
    
    fn region_query(&self, data: &[Point], point_idx: usize) -> Vec<usize> {
        let point = data[point_idx];
        data.iter()
            .enumerate()
            .filter(|(i, p)| {
                *i != point_idx && p.distance(&point) <= self.eps
            })
            .map(|(i, _)| i)
            .collect()
    }
    
    fn expand_cluster(
        &self, 
        data: &[Point], 
        point_idx: usize, 
        neighbors: Vec<usize>,
        cluster_id: usize,
        clusters: &mut Vec<Option<usize>>,
        point_types: &mut Vec<PointType>,
    ) {
        clusters[point_idx] = Some(cluster_id);
        point_types[point_idx] = PointType::Core;
        
        let mut seeds = VecDeque::from(neighbors);
        while let Some(current_idx) = seeds.pop_front() {
            if point_types[current_idx] == PointType::Noise {
                clusters[current_idx] = Some(cluster_id);
                point_types[current_idx] = PointType::Border;
                continue;
            }
            
            if clusters[current_idx].is_some() {
                continue;
            }
            
            clusters[current_idx] = Some(cluster_id);
            
            let new_neighbors = self.region_query(data, current_idx);
            
            if new_neighbors.len() >= self.min_points {
                point_types[current_idx] = PointType::Core;
                for &neighbor_idx in &new_neighbors {
                    if clusters[neighbor_idx].is_none() || point_types[neighbor_idx] == PointType::Noise {
                        seeds.push_back(neighbor_idx);
                    }
                }
            } else {
                point_types[current_idx] = PointType::Border;
            }
        }
    }
    
    fn fit(&self, data: &[Point]) -> (Vec<Option<usize>>, Vec<PointType>) {
        let n = data.len();
        let mut clusters: Vec<Option<usize>> = vec![None; n];
        let mut point_types: Vec<PointType> = vec![PointType::Unclassified; n];
        
        let mut cluster_id = 0;
        
        for i in 0..n {
            if clusters[i].is_some() {
                continue;
            }
            
            let neighbors = self.region_query(data, i);
            
            if neighbors.len() < self.min_points {
                point_types[i] = PointType::Noise;
                continue;
            }
            
            cluster_id += 1;
            self.expand_cluster(data, i, neighbors, cluster_id, &mut clusters, &mut point_types);
        }
        
        (clusters, point_types)
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
    
    let test_params = vec![
        (1.5, 2),
        (2.0, 2),
        (2.5, 2),
        (3.0, 2),
    ];
    
    for (eps, min_points) in test_params {
        println!("\nRunning DBSCAN with eps = {}, min_points = {}", eps, min_points);
        
        let dbscan = DBSCAN::new(eps, min_points);
        let (clusters, point_types) = dbscan.fit(&data);
        
        let mut unique_clusters = HashSet::new();
        let mut noise_count = 0;
        
        for cluster in &clusters {
            match cluster {
                Some(id) => { unique_clusters.insert(id); },
                None => noise_count += 1,
            }
        }
        
        println!("Found {} clusters and {} noise points", unique_clusters.len(), noise_count);
        
        let mut cluster_map: HashMap<Option<usize>, Vec<(f64, f64, PointType)>> = HashMap::new();
        
        for (i, cluster) in clusters.iter().enumerate() {
            let point = data[i];
            let point_type = point_types[i];
            cluster_map.entry(*cluster).or_insert_with(Vec::new)
                .push((point.x, point.y, point_type));
        }
        
        if let Some(points) = cluster_map.get(&None) {
            println!("Noise points: {:?}", points.iter()
                .map(|(x, y, _)| format!("({:.1},{:.1})", x, y))
                .collect::<Vec<_>>());
        }
        
        for cluster_id in unique_clusters {
            if let Some(points) = cluster_map.get(&Some(*cluster_id)) {
                println!("Cluster {}: {:?}", cluster_id, points.iter()
                    .map(|(x, y, pt)| format!("({:.1},{:.1}):{}", x, y, 
                        match pt {
                            PointType::Core => "Core",
                            PointType::Border => "Border",
                            _ => "Unknown",
                        }
                    ))
                    .collect::<Vec<_>>());
            }
        }
    }
}
