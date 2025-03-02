use std::ops::RangeBounds;
use std::ops::RangeInclusive;
use std::str::FromStr;

use winnow::ascii::alpha1;
use winnow::ascii::dec_uint;
use winnow::ascii::line_ending;
use winnow::ascii::multispace0;
use winnow::ascii::multispace1;
use winnow::ascii::space0;
use winnow::ascii::space1;
use winnow::ascii::take_escaped;
use winnow::combinator::alt;
use winnow::combinator::delimited;
use winnow::combinator::not;
use winnow::combinator::opt;
use winnow::combinator::repeat;
use winnow::combinator::separated;
use winnow::combinator::terminated;
use winnow::error::ParserError;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::stream::Compare;
use winnow::stream::Stream;
use winnow::stream::StreamIsPartial;
use winnow::token::any;
use winnow::token::none_of;
use winnow::token::one_of;
use winnow::token::take_until;
use winnow::token::take_while;
use winnow::Parser;

#[derive(PartialEq, Debug)]
pub struct NodesRange(RangeInclusive<usize>);
#[derive(PartialEq, Debug)]
pub struct NodesList(Vec<usize>);

#[derive(PartialEq, Debug)]
pub enum Nodes {
    Range(NodesRange),
    List(NodesList)
}

#[derive(PartialEq, Debug)]
pub struct Edge {
    id: usize,
    src: usize,
    tgt: usize
}

#[derive(PartialEq, Debug)]
pub struct Edges(Vec<Edge>);

#[derive(PartialEq, Debug)]
pub struct Date(String);

#[derive(PartialEq, Debug)]
pub struct Comments(String);

#[derive(PartialEq, Debug)]
pub struct Author(String);

#[derive(PartialEq, Debug, Clone)]
pub enum PropertyType {
    Bool,
    Color,
    Double,
    Graph,
    Int,
    Layout,
    String,
    Size
}

#[derive(PartialEq, Debug, Clone)]
pub struct Property {
    graph_id: usize,
    name: String,
    r#type: PropertyType,
    node_default: String,
    edge_default: String,

    nodes_property: Vec<NodeProperty>
}

#[derive(PartialEq, Debug, Clone)]
pub struct Attribute {
    r#type: PropertyType,
    name: String,
    value: String
}

#[derive(PartialEq, Debug, Clone)]
pub struct Attributes(Vec<Attribute>);
#[derive(PartialEq, Debug, Clone)]
pub struct Properties(Vec<Property>);



#[derive(PartialEq, Debug, Clone)]
pub struct NodeProperty{
    id: usize,
    value: String
}

pub struct Graph {
    version: String,

    author: Option<Author>,
    comments: Option<Comments>,
    date: Option<Date>,

    nodes: Nodes,
    edges: Edges,

    properties: Properties,
    attributes: Attributes
}


impl FromStr for Graph {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        graph.parse(s)
            .map_err(|e| e.to_string())
    }
}


fn parse_tag<Input, Content, Error, Inner>(tag: &'static str, mut f: Inner) -> impl Parser<Input, Content, Error>
where 
Input: Stream + StreamIsPartial + Compare<char> + for<'a> Compare<&'a str>,
Inner: Parser<Input, Content, Error>,
Error: ParserError<Input>, 
<Input as Stream>::Token: AsChar
{
    move |input: &mut Input| {

        let _ = ('(', space0, tag, space1).parse_next(input)?; 
        let res = f.parse_next(input)?; 
        let _ = (space0, ')').parse_next(input)?;
        Ok(res)
    }
}

fn parse_range(input: &mut &str) -> ModalResult<RangeInclusive<usize>> {
    let start = dec_uint(input)?;
    "..".parse_next(input)?;
    let end = dec_uint(input)?;

    Ok(RangeInclusive::new(start, end))
}

fn parse_comma(input: &mut &str) -> ModalResult<()> {
    (space0, ',', space0)
        .value(())
        .parse_next(input)
}

fn parse_ids_list(input: &mut &str) -> ModalResult<Vec<usize>> {
    separated(1.., dec_uint::<_, usize, _>, space1)
        .parse_next(input)
}

fn nodes_range(input: &mut &str) -> ModalResult<NodesRange> {
    parse_tag("nodes", parse_range)
        .map(|r| NodesRange(r))
        .parse_next(input)
}

fn nodes_list(input: &mut &str) -> ModalResult<NodesList> {
    parse_tag("nodes", parse_ids_list)
        .map(|l| NodesList(l))
        .parse_next(input)
}


fn nodes(input: &mut &str) -> ModalResult<Nodes> {
    let nb_nodes: Option<usize> = opt(terminated(parse_tag("nb_nodes", dec_uint), multispace1)).parse_next(input)?;

    dbg!(&nb_nodes);
    // TODO handle it to improve reading

    let _ = terminated(opt((';', take_until(.., '\n'))), multispace0).parse_next(input)?;

    alt((
        nodes_range.map(|n| Nodes::Range(n)),
        nodes_list.map(|l| Nodes::List(l))
    ))
    .parse_next(input)
}

fn parse_string(input: &mut &str) -> ModalResult<String> {
    alt(("\"\"".map(|_| "".to_owned()),
        delimited(
        '"', 
        take_escaped(alt((alpha1, space1, take_while(1.., |c| c!= '"'))),
         '\\', 
         '"'), 
         '"'
        )
        .map(|s: &str| s.to_owned())
    )).parse_next(input)
}

fn date(input: &mut &str) -> ModalResult<Date> {
    parse_tag("date", parse_string)
        .map(|s| Date(s))
        .parse_next(input)
}

fn comments(input: &mut &str) -> ModalResult<Comments> {
    parse_tag("comments", parse_string)
        .map(|s| Comments(s))
        .parse_next(input)
}

fn author(input: &mut &str) -> ModalResult<Author> {
    parse_tag("author", parse_string)
        .map(|s| Author(s))
        .parse_next(input)
}

fn edge(input: &mut &str) -> ModalResult<Edge> {

    fn edge_inner(input: &mut &str) -> ModalResult<(usize, usize, usize)> {
        let id = dec_uint.parse_next(input)?;
        space1.parse_next(input)?;
        let src = dec_uint.parse_next(input)?;
        space1.parse_next(input)?;
        let tgt = dec_uint.parse_next(input)?;

        Ok((id, src, tgt))
    }

    parse_tag("edge", edge_inner)
        .map(|(id, src, tgt)| {
            Edge{
                id, src, tgt
            }
        })
        .parse_next(input)
}

fn nb_edges(input: &mut &str) -> ModalResult<usize> {
    parse_tag("nb_edges", dec_uint::<_, usize, _>)
        .parse_next(input)
}


fn edges(input: &mut &str) -> ModalResult<Edges> {
    let count = opt(delimited(multispace0, nb_edges, multispace0))
        .parse_next(input)?;

    dbg!(count);
    let _ = terminated(opt((';', take_until(.., '\n'))), multispace0).parse_next(input)?;


    let edges = if let Some(count) = count {
        separated::<_, Edge, Vec<Edge>, _,_,_,_>(count, edge, multispace0).parse_next(input)?
    } else {
        separated(.., edge, multispace0).parse_next(input)?
    };
    
    Ok(Edges(edges))
}

fn property_type(input: &mut &str) -> ModalResult<PropertyType> {
    alt((
        "color".value(PropertyType::Color),
        "double".value(PropertyType::Double),
        "string".value(PropertyType::String),
        "int".value(PropertyType::Int),
        "layout".value(PropertyType::Layout),
        "graph".value(PropertyType::Graph),
        "bool".value(PropertyType::Bool),
        "size".value(PropertyType::Size),
    )).parse_next(input)
}

fn property_default(input: &mut &str) -> ModalResult<(String, String)>  {
    fn default_inner(input: &mut &str) -> ModalResult<(String, String)> {
        let node = terminated(parse_string, multispace1).parse_next(input)?;
        let edge = terminated(parse_string, multispace0).parse_next(input)?;

        Ok((node, edge))
    }

    parse_tag("default", default_inner).parse_next(input)
}

fn property_for_node(input: &mut &str) -> ModalResult<NodeProperty> {
    fn for_node_inner(input: &mut &str) -> ModalResult<NodeProperty> {
        let id: usize = terminated(dec_uint, multispace1).parse_next(input)?;
        let value: String = terminated(parse_string, multispace0).parse_next(input)?;
        Ok(NodeProperty {
            id, value
        })
    }

    parse_tag("node", for_node_inner)
        .parse_next(input)
}


fn property(input: &mut &str) -> ModalResult<Property> {
    fn property_inner(input: &mut &str) -> ModalResult<Property> {
        let graph_id: usize = delimited(multispace0, dec_uint, multispace1).parse_next(input)?;
        let r#type = terminated(property_type, multispace1).parse_next(input)?;
        let name = terminated(parse_string, multispace1).parse_next(input)?;
        dbg!(&input);

        let default = terminated(property_default, multispace1).parse_next(input)?;

        dbg!(&input);
        let nodes_property: Option<Vec<NodeProperty>> = dbg!(opt(terminated(repeat(.., terminated(property_for_node, multispace0)), multispace0))
            .parse_next(input))?;
        Ok(Property { graph_id, name, r#type, node_default: default.0, edge_default: default.1, nodes_property: nodes_property.unwrap_or_default() })
    }

    parse_tag("property", property_inner).parse_next(input)
}

fn properties(input: &mut &str) -> ModalResult<Properties> {
    repeat(.., terminated(property, multispace0))
        .map(|prop| Properties(prop))
        .parse_next(input)
}

fn attribute(input: &mut &str) -> ModalResult<Attribute> {
    let (r#type, name, value) = delimited(
        (multispace0, '(', multispace0),
        (
            terminated(property_type, multispace1),
            terminated(parse_string, multispace1),
            parse_string,
        ),
        (multispace0, ')', multispace0),
    ).parse_next(input)?;

    Ok(Attribute{r#type, name, value})
}

fn attributes(input: &mut &str) -> ModalResult<Attributes> {
    fn attributes_inner(input: &mut &str) -> ModalResult<Attributes> {
        let graph_id: usize = delimited(multispace0, dec_uint, multispace1).parse_next(input)?;
    
        repeat(.., terminated(attribute, multispace0))
            .map(|attr| Attributes(attr))
            .parse_next(input)    
    }

    parse_tag("graph_attributes", attributes_inner).parse_next(input)


    
}

fn graph(input: &mut &str) -> ModalResult<Graph> {

    fn inner_graph(input: &mut &str) -> ModalResult<Graph> {
        dbg!(&input);
        let version = dbg!(terminated(parse_string, multispace0).parse_next(input))?;
        
        // TODO handle random ordering
        let date = dbg!(opt(terminated(date, multispace0)).parse_next(input))?;
        let comments = dbg!(opt(terminated(comments, multispace0)).parse_next(input))?;

        let nodes = dbg!(terminated(nodes, multispace0).parse_next(input))?;
        let edges = dbg!(terminated(edges, multispace0).parse_next(input))?;

        // TODO check the edges are valid in comparison to nodes

        let properties = dbg!(terminated(properties, multispace0).parse_next(input))?;
        let attributes = dbg!(terminated(attributes, multispace0).parse_next(input))?;
        
        Ok(Graph{
            version,
            nodes,
            edges,

            properties,
            attributes,

            author: None,
            comments,
            date
        })
    }

    terminated(parse_tag("tlp", inner_graph), multispace0).parse_next(input)
}

#[cfg(test)]
mod test {
    use crate::{edge, graph, nodes_list, nodes_range, parse_range, parse_string, property, property_default, property_for_node, property_type, Edge, NodesList, NodesRange};

    #[test]
    fn test_nodes_list() {
        let  mut repr = "(nodes 0 1 2 3 4 5 )";
        let nodes: NodesList = nodes_list(&mut repr).unwrap();
        assert_eq!(
            nodes,
            NodesList((0..=5).collect())
        );
    }

    #[test]
    fn test_property_empty() {
        let reprs = &[
            r#"(property  0 double "viewLabelBorderWidth"
(default "1" "1")
)"#,
            r#"(property  0 string "viewLabel"
(default "" "")
)"#,

            r#"(property  0 layout "viewLayout"
(default "(0,0,0)" "()")
)"#,

            r#"(property  0 double "viewBorderWidth"
(default "0" "0")
)"#

];
        for repr in reprs.iter() {
            dbg!(&repr);
            let prop = property(&mut repr.clone()).unwrap();
            dbg!("ok");
        }
    }

    #[test]
    fn test_node_property() {
        property_for_node(&mut r#"(node 0 "(11,-6,0)\")"# ).unwrap();
            
    }

    #[test]
    fn test_property_with_nodes() {
        let reprs = &[
            r#"(property  0 layout "viewLayout"
(default "(0,0,0)" "()")
(node 0 "(11,-6,0)")
(node 1 "(-2,-15,0)")
(node 2 "(6,10,0)")
(node 3 "(-15,-7,0)")
(node 4 "(-11,10,0)")
)   "#

];
        for repr in reprs.iter() {
            let prop = property(&mut repr.clone()).unwrap();
        }
    }

    #[test]
    fn test_types() {
        let mut reprs = [
            "int", "color", "double", "string"
        ];

        for repr in &mut reprs {
            let t = property_type(repr).unwrap();
        }
    }

    #[test]
    fn test_property_default() {
        let mut reprs = [
            r#"(default "1" "1")"#,
            r#"(default "(0,0,0,255)" "(0,0,0,255)")",
            r#"(default "" "")"#,
            r#"(default "18" "18")"#,
            r#"(default "(0,0,0)" "()")"#
        ];

        for repr in &mut reprs {
            let t = property_default(dbg!(repr)).unwrap();
        }
    }

    #[test]
    fn test_nodes_range() {
        let mut repr = "(nodes 0..5)";
        let nodes: NodesRange = nodes_range(&mut repr).unwrap();
        assert_eq!(
            nodes,
            NodesRange((0..=5))
        );
    }


    #[test]
    fn test_edge() {
        let mut repr = "(edge 0 1 2)";
        let e = edge(&mut repr).unwrap();
        assert_eq!(
            e,
            Edge{id: 0, src: 1, tgt: 2}
        );
    }

    #[test]
    fn test_string() {
        let mut repr = dbg!(r#""string""#);
        let s = parse_string(&mut repr).unwrap();
        assert_eq!(
            s,
            "string"
        )
    }


    #[test]
    fn test_strings() {
        let reprs = [
            r#""stri ng""#,
            r#""stri ng,;:!?./ù*%µ^$¨£""#,
            r#""""#,
            r#""a""#,
            r#""a1""#,
            r#""1""#,
            "\"(0,0,0)\""
        ];
        for repr in &reprs {
            dbg!(&repr);
            let s = parse_string(&mut repr.clone()).unwrap();
            assert_eq!(
                &s,
                &repr[1..repr.len()-1]
            )
        }
    }


    #[test]
    fn test_graph() {
        let mut repr = r#"(tlp "2.0"
(nodes 0 1 2)
(edge 0 1 0)
(edge 1 0 2)
)"#;
        let g  = graph(&mut repr).unwrap();
    }
}