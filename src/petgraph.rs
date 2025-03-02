use std::collections::HashMap;

use petgraph;

use crate::Graph;

impl Graph {
    pub fn into_petgraph(&self) -> petgraph::Graph<usize, usize> {
        let mut g = petgraph::Graph::<usize, usize>::new();

        let mut node_id_to_idx = HashMap::new();

        for  n in self.nodes_iter() {
            let idx = g.add_node(n);
            node_id_to_idx.insert(n, idx);
        }

        for e in self.edges_iter() {
            let id_src = &e.src;
            let id_tgt = &e.tgt;
            g.add_edge(
                *node_id_to_idx.get(id_src).unwrap(),
                *node_id_to_idx.get(id_tgt).unwrap(),
                e.id,
            );
        }

        g

    }
}