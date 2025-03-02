use std::str::FromStr;

use graphtlp::Graph;

#[test]
fn load_complete() {
    let content = std::fs::read_to_string("data/complete.tlp").unwrap();
    let g = Graph::from_str(&content).unwrap();

    #[cfg(feature="petgraph")]
    {
        use graphtlp::petgraph;
        let p = g.into_petgraph();
    }
}


#[test]
fn load_grid() {
    let content = std::fs::read_to_string("data/grid.tlp").unwrap();
    let g = Graph::from_str(&content).unwrap();

    #[cfg(feature="petgraph")]
    {
        let p = g.into_petgraph();
    }
}