use ruin_ecs::physics_2d::AABB;

type Index = usize;

pub struct Node<T> {
    pub aabb: AABB,
    pub left: Option<Index>,
    pub right: Option<Index>,
    pub user_data: Option<Vec<T>>,
}

pub struct BVH<T> {
    pub nodes: Vec<Node<T>>,
    pub root: Option<Index>,
}

impl<T: Clone> BVH<T> {
    pub fn build(objects: &mut [(AABB, T)], max_leaf_size: usize) -> Self {
        let mut bvh = Self {
            nodes: Vec::new(),
            root: None,
        };

        if !objects.is_empty() {
            bvh.root = Some(bvh.build_recursive(objects, 0, objects.len(), max_leaf_size));
        }

        bvh
    }

    fn build_recursive(
        &mut self,
        objects: &mut [(AABB, T)],
        start: usize,
        end: usize,
        max_nodes_on_leaf: usize,
    ) -> Index {
        let mut aabb = objects[start].0;
        for (other, _) in &objects[start + 1..end] {
            aabb.merge(other)
        }

        if end - start <= max_nodes_on_leaf {
            self.nodes.push(Node {
                aabb,
                left: None,
                right: None,
                user_data: Some(objects[start..end].iter().map(|(_, t)| t.clone()).collect()),
            });
            return self.nodes.len() - 1;
        }

        let axis = aabb.longest_axis();
        let mid = (start + end) / 2;

        // Partition in-place instead of full sorting
        objects[start..end].select_nth_unstable_by(mid - start, |(aabb_a, _), (aabb_b, _)| {
            let ca = aabb_a.center();
            let cb = aabb_b.center();
            match axis {
                0 => ca.x.partial_cmp(&cb.x).unwrap(),
                1 => ca.y.partial_cmp(&cb.y).unwrap(),
                _ => std::cmp::Ordering::Equal,
            }
        });

        let left_index = self.build_recursive(objects, start, mid, max_nodes_on_leaf);
        let right_index = self.build_recursive(objects, mid, end, max_nodes_on_leaf);

        self.nodes.push(Node {
            aabb: aabb,
            left: Some(left_index),
            right: Some(right_index),
            user_data: None,
        });

        self.nodes.len() - 1
    }
}
