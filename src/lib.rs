use std::fmt::Debug;
use std::ops::Deref;
use std::ops::RangeInclusive;
use std::str::FromStr;

use winnow::ascii::alpha1;
use winnow::ascii::dec_uint;
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
use winnow::token::take_until;
use winnow::token::take_while;
use winnow::Parser;
use winnow::Result;

#[derive(PartialEq, Debug, Clone)]
pub struct IdsRange(RangeInclusive<usize>);
#[derive(PartialEq, Debug, Clone)]
pub struct IdsList(Vec<usize>);


#[derive(PartialEq, Debug, Clone)]
pub enum IdsBloc {
    Range(IdsRange),
    List(IdsList)
}

#[derive(PartialEq, Debug, Clone)]
pub struct Ids(Vec<IdsBloc>);



impl IdsRange {
    pub fn len(&self) -> usize {
        self.0.clone().count()
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.0.clone().collect()
    }
}

impl IdsList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.0.clone()
    }
}

impl IdsBloc {
    pub fn len(&self) -> usize {
        match self {
            Self::Range(r) => r.len(),
            Self::List(l) => l.len()

        }
    }

    // it would be better to use an iterator
    #[cfg(test)]
    pub fn to_vec(&self) -> Vec<usize> {
        match self {
            Self::Range(r) => r.to_vec(),
            Self::List(l) => l.to_vec()
        }
    }
}

impl Ids {
    pub fn len(&self) -> usize {
        self.0.iter().map(IdsBloc::len).sum()
    }

    #[cfg(test)]
    // it would be better to use an iterator
    pub fn to_vec(&self) -> Vec<usize> {
        self.0.iter()
            .map(|bloc| bloc.to_vec())
            .flatten()
            .collect()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct NodesIds(Ids);
#[derive(PartialEq, Debug, Clone)]
pub struct EdgesIds(Ids);

impl Deref for NodesIds {
    type Target = Ids;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for EdgesIds {
    type Target = Ids;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
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


#[derive(PartialEq, Debug)]
struct Cluster {
    id: usize,
    nodes: NodesIds,
    edges: EdgesIds,

    clusters: Vec<Cluster>
}

#[derive(PartialEq, Debug)]
pub struct Clusters(Vec<Cluster>);

#[derive(PartialEq, Debug)]
pub struct Graph {
    version: String,

    author: Option<Author>,
    comments: Option<Comments>,
    date: Option<Date>,

    nodes: NodesIds,
    edges: Edges,

    properties: Option<Properties>,
    attributes: Option<Attributes>,

    clusters: Option<Clusters>,
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
Error: ParserError<Input> + Debug, 
<Input as Stream>::Token: AsChar,
Content: Debug
{
    move |input: &mut Input| {

        let _ = (('(', space0, tag, space1).parse_next(input))?; 
        let res = (f.parse_next(input))?; 
        let _ = ((space0, opt(')')).parse_next(input))?;
        Ok(res)
    }
}



fn parse_comma(input: &mut &str) -> ModalResult<()> {
    (space0, ',', space0)
        .value(())
        .parse_next(input)
}

fn parse_ids_range(input: &mut &str) -> ModalResult<IdsRange> {
    let start = dec_uint(input)?;
    "..".parse_next(input)?;
    let end = dec_uint(input)?;

    Ok(IdsRange(RangeInclusive::new(start, end)))
}

fn parse_ids_list(input: &mut &str) -> ModalResult<IdsList> {
    separated(1.., terminated( dec_uint::<_, usize, _>, not("..")), multispace1)
        .map(|l| IdsList(l))
        .parse_next(input)
}

fn parse_ids_bloc(input: &mut &str) -> ModalResult<IdsBloc> {
    alt((
        parse_ids_range.map(|r| IdsBloc::Range(r)),
        parse_ids_list.map(|l| IdsBloc::List(l))
    )).parse_next(input)
}

fn parse_ids(input: &mut &str) -> ModalResult<Ids> {
    separated(1.., parse_ids_bloc, multispace1)
        .map(|ids| Ids(ids))
        .parse_next(input)
}

fn nodes_ids(input: &mut &str) -> ModalResult<NodesIds> {
    parse_tag("nodes", parse_ids)
    .map(|ids| NodesIds(ids))
    .parse_next(input)
}

fn edges_ids(input: &mut &str) -> ModalResult<EdgesIds> {
    parse_tag("edges", parse_ids)
    .map(|ids| EdgesIds(ids))
    .parse_next(input)
}


fn cluster(input: &mut &str) -> ModalResult<Cluster> {

    fn cluster_inner(input: &mut &str) -> ModalResult<Cluster> {
        let id: usize = terminated(dec_uint, multispace1).parse_next(input)?;
        let nodes = terminated(nodes_ids, multispace1).parse_next(input)?;
        let edges = terminated(edges_ids, multispace0).parse_next(input)?;

        let clusters = repeat(.., cluster).parse_next(input)?;

        let _ = multispace0.parse_next(input)?;
        Ok(Cluster { id, nodes, edges, clusters})
    }

    parse_tag("cluster", cluster_inner)
        .parse_next(input)
}

fn clusters(input: &mut &str) -> ModalResult<Clusters> {
    separated(1.., cluster, multispace1)
        .map(|c| Clusters(c))
        .parse_next(input)
}

fn nodes_amount_and_ids(input: &mut &str) -> ModalResult<NodesIds> {
    let nb_nodes: Option<usize> = opt(terminated(parse_tag("nb_nodes", dec_uint), multispace1)).parse_next(input)?;

    // TODO handle it to improve reading

    let _ = terminated(opt((';', take_until(.., '\n'))), multispace0).parse_next(input)?;

    let nodes = nodes_ids.parse_next(input)?;

    if let Some(nb_nodes) = nb_nodes {
        if nodes.len() != nb_nodes {
            eprintln!("[WARNING] Expected {nb_nodes} but obtained {}", nodes.len());
        }
    }
    Ok(nodes)
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
        .map(Date)
        .parse_next(input)
}

fn comments(input: &mut &str) -> ModalResult<Comments> {
    parse_tag("comments", parse_string)
        .map(Comments)
        .parse_next(input)
}

fn author(input: &mut &str) -> ModalResult<Author> {
    parse_tag("author", parse_string)
        .map(Author)
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

    let _ = terminated(opt((';', take_until(.., '\n'))), multispace0).parse_next(input)?;

/*
    // XXX for an unknwon reason the number of edges may not work
    let edges = if let Some(count) = count {
        dbg!("Try to parse nb edges: ", count);
        separated::<_, Edge, Vec<Edge>, _,_,_,_>(count, edge, multispace0).parse_next(input)?
    } else {
       dbg!("Unknow amount of edges");
        separated(.., edge, multispace0).parse_next(input)?
    };
*/
    let edges: Vec<Edge> = separated(.., edge, multispace0).parse_next(input)?;

    if let Some(count) = count {
        if edges.len() != count {
            eprintln!("[WARNING] {count} edges expected, but {} obtained.", edges.len());
        }
    }
    
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

        let default = terminated(property_default, multispace1).parse_next(input)?;

        let nodes_property: Option<Vec<NodeProperty>> = opt(terminated(repeat(.., terminated(property_for_node, multispace0)), multispace0))
            .parse_next(input)?;
        Ok(Property { graph_id, name, r#type, node_default: default.0, edge_default: default.1, nodes_property: nodes_property.unwrap_or_default() })
    }

    parse_tag("property", property_inner).parse_next(input)
}

fn properties(input: &mut &str) -> ModalResult<Properties> {
    repeat(.., terminated(property, multispace0))
        .map(Properties)
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
            .map(Attributes)
            .parse_next(input)    
    }

    parse_tag("graph_attributes", attributes_inner).parse_next(input)
}

fn graph(input: &mut &str) -> ModalResult<Graph> {

    fn inner_graph(input: &mut &str) -> ModalResult<Graph> {
        let version = terminated(parse_string, multispace0).parse_next(input)?;
        
        // TODO handle random ordering
        let date = (opt(terminated(date, multispace0)).parse_next(input))?;
        let comments = (opt(terminated(comments, multispace0)).parse_next(input))?;

        let nodes = (terminated(nodes_amount_and_ids, multispace0).context(winnow::error::StrContext::Label("Nodes parsing")).parse_next(input))?;
        let edges = (terminated(edges, multispace0).context(winnow::error::StrContext::Label("Edges parsing")).parse_next(input))?;

        // TODO check the edges are valid in comparison to nodes

        // TODO handle a different ordering
        let clusters = opt(terminated(clusters, multispace0)).parse_next(input)?;
        let properties = opt(terminated(properties, multispace0)).parse_next(input)?;
        let attributes = opt(terminated(attributes, multispace0)).parse_next(input)?;
        
        Ok(Graph{
            version,
            nodes,
            edges,

            properties,
            attributes,
            clusters,

            author: None,
            comments,
            date
        })
    }

    terminated(parse_tag("tlp", inner_graph), multispace0).parse_next(input)
}

#[cfg(test)]
mod test {
    use winnow::Parser;

    use crate::{cluster, edge, edges_ids, graph, nodes_ids, parse_ids, parse_ids_bloc, parse_ids_list, parse_ids_range, parse_string, property, property_default, property_for_node, property_type, Edge, IdsBloc, IdsList, IdsRange, NodesIds};

    #[test]
    fn test_nodes_list() {
        let  mut repr = "(nodes 0 1 2 3 4 5 )";
        let nodes: NodesIds = nodes_ids(&mut repr).unwrap();
        assert_eq!(
            nodes.to_vec(),
            (0..=5).collect::<Vec<usize>>()
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
            let prop = property(&mut repr.clone()).unwrap();
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
            let t = property_default(repr).unwrap();
        }
    }

    #[test]
    fn test_nodes_range() {
        let mut repr = "(nodes 0..5)";
        let nodes = nodes_ids(&mut repr).unwrap();
        assert_eq!(
            nodes.to_vec(),
            IdsRange(0..=5).to_vec()
        );
    }


    #[test]
    fn test_edges_range() {
        let mut repr = "(edges 0..70708)";
        let edges = edges_ids(&mut repr).unwrap();
        assert_eq!(
            edges.to_vec(),
            IdsRange(0..=70708).to_vec()
        );
    }

    #[test]
    fn test_ids() {
        parse_ids_list.parse(&mut "37830 37829").unwrap();
        parse_ids_range.parse(&mut "37830..37829").unwrap();
        parse_ids_bloc.parse(&mut "37830 37829").unwrap();
        parse_ids_bloc.parse(&mut "37830..37829").unwrap();
        parse_ids.parse(&mut "37830..37829 37830..37829").unwrap();
        parse_ids.parse(&mut "37830 37829 37830 37829").unwrap();
        parse_ids.parse(&mut "37830 37829..37830 37829").unwrap();
    }

    #[test]
    fn test_nodes_list2() {
        let mut repr = "37830 37829 37828 37827 37826 37825 37824 37823 37822 37821 37820 37819 37818 37817 37816 37815 37814 37813 37812 37811 37810 37809 37808 37807 37806 37805 37804 37803 37802 37801 37800 37799 37798 37797 37796 37795 37794 37793 37792 37791 37790 37789 37788 37787 37786 37785 37784 37783 37782 37781 37780 37779 37778 37777 37776 37775 37774 37773 37772 37771 37770 37769 37768 37767 37766 37765 37764 37763 37762 37761 37760 37759 37758 37757 37756 37755 37754 37753 37752 37751 37750 37749 37748 37747 37746 37745 37744 37743 37742 37741 37740 37739 37738 37737 37736 37735 37734 37733 37732 37731 37730 37729 37728 37727 37726 37725 37724 37723 37722 37721 37720 37719 37718 37717 37716 37715 37714 37713 37712 37711 37710 37709 37708 37707 37706 37705 37704 37703 37702 37701 37700 37699 37698 37697 37696 37695 37694 37693 37692 37691 37690 37689 37688 37687 37686 37685 37684 37683 37682 37681 37680 37679 37678 37677 37676 37675 37674 37673 37672 37671 37670 37669 37668 37667 37666 37665 37664 37663 37662 37661 37660 37659 37658 37657 37656 37655 37654 37653 37652 37651 37650 37649 37648 37647 37646 37645 37644 37643 37642 37641 37640 37639 37638 37637 37636 37635 37634 37633 37632 37631 37630 37629 37628 37627 37626 37625 37624 37623 37622 37621 37620 37619 37618 37617 37616 37615 37614 37613 37612 37611 37610 37609 37608 37607 37606 37605 37604 37603 37602 37601 37600 37599 37598 37597 37596 37595 37594 37593 37592 37591 37590 37589 37588 37587 37586 37585 37584 37583 37582 37581 37580 37579 37578 37577 37576 37575 37574 37573 37572 37571 37570 37569 37568 37567 37566 37565 37564 37563 37562 37561 37560 37559 37558 37557 37556 37555 37554 37553 37552 37551 37550 37549 37548 37547 37546 37545 37544 37543 37542 37541 37540 37539 37538 37537 37536 37535 37534 37533 37532 37531 37530 37529 37528 37527 37526 37525 37524 37523 37522 37521 37520 37519 37518 37517 37516 37515 37514 37513 37512 37511 37510 37509 37508 37507 37506 37505 37504 37503 37502 37501 37500 37499 37498 37497 37496 37495 37494 37493 37492 37491 37490 37489 37488 37487 37486 37485 37484 37483 37482 37481 37480 37479 37478 37477 37476 37475 37474 37473 37472 37471 37470 37469 37468 37467 37466 37465 37464 37463 37462 37461 37460 37459 37458 37457 37456 37455 37454 37453 37452 37451 37450 37449 37448 37447 37446 37445 37444 37443 37442 37441 37440 37439 37438 37437 37436 37435 37434 37433 37432 37431 2002..2820 37314 2821 37315 2822 37316 2823 37317 2824 37318 2825 37319 2826 37320 2827 37321 2828 37322 2829 37323 2830 37324 2831 37325 2832 37326 2833 37327 2834 37328 2835 37329 2836 37330 2837 37331 2838 37332 2839 37333 2840 37334 2841 37335 2842 37336 2843 37337 2844 37338 2845 37339 2846 37340 2847 37341 2848 37342 2849 37343 2850 37344 2851 37345 2852 37346 2853 37347 2854 37348 2855 37349 2856 37350 2857 37351 2858 37352 2859 37353 2860 37354 2861 37355 2862 37356 2863 37357 2864 37358 2865 37359 2866 37360 2867 37361 2868 37362 2869 37363 2870 37364 2871 37365 2872 37366 2873 37367 2874 37368 2875 37369 2876 37370 2877 37371 2878 37372 2879 37373 2880 37374 2881 37375 2882 37376 2883 37377 2884 37378 2885 37379 2886 37380 2887 37381 2888 37382 2889 37383 2890 37384 2891 37385 2892 37386 2893 37387 2894 37388 2895 37389 2896 37390 2897 37391 2898 37392 2899 37393 2900 37394 2901 37395 2902 37396 2903 37397 2904 37398 2905 37399 2906 37400 2907 37401 2908 37402 2909 37403 2910 37404 2911 37405 2912 37406 2913 37407 2914 37408 2915 37409 2916 37410 2917 37411 2918 37412 2919 37413 2920 37414 2921 37415 2922 37416 2923 37417 2924 37418 2925 37419 2926 37420 2927 37421 2928 37422 2929 37423 2930 37424 2931 37425 2932 37426 2933 37427 2934 37428 2935 37429 2936 37430 2937";
        let list = parse_ids(&mut repr).unwrap();
        
        dbg!(&repr);
        assert_eq!(repr.len(), 0);

        let mut repr = "(nodes 37830 37829 37828 37827 37826 37825 37824 37823 37822 37821 37820 37819 37818 37817 37816 37815 37814 37813 37812 37811 37810 37809 37808 37807 37806 37805 37804 37803 37802 37801 37800 37799 37798 37797 37796 37795 37794 37793 37792 37791 37790 37789 37788 37787 37786 37785 37784 37783 37782 37781 37780 37779 37778 37777 37776 37775 37774 37773 37772 37771 37770 37769 37768 37767 37766 37765 37764 37763 37762 37761 37760 37759 37758 37757 37756 37755 37754 37753 37752 37751 37750 37749 37748 37747 37746 37745 37744 37743 37742 37741 37740 37739 37738 37737 37736 37735 37734 37733 37732 37731 37730 37729 37728 37727 37726 37725 37724 37723 37722 37721 37720 37719 37718 37717 37716 37715 37714 37713 37712 37711 37710 37709 37708 37707 37706 37705 37704 37703 37702 37701 37700 37699 37698 37697 37696 37695 37694 37693 37692 37691 37690 37689 37688 37687 37686 37685 37684 37683 37682 37681 37680 37679 37678 37677 37676 37675 37674 37673 37672 37671 37670 37669 37668 37667 37666 37665 37664 37663 37662 37661 37660 37659 37658 37657 37656 37655 37654 37653 37652 37651 37650 37649 37648 37647 37646 37645 37644 37643 37642 37641 37640 37639 37638 37637 37636 37635 37634 37633 37632 37631 37630 37629 37628 37627 37626 37625 37624 37623 37622 37621 37620 37619 37618 37617 37616 37615 37614 37613 37612 37611 37610 37609 37608 37607 37606 37605 37604 37603 37602 37601 37600 37599 37598 37597 37596 37595 37594 37593 37592 37591 37590 37589 37588 37587 37586 37585 37584 37583 37582 37581 37580 37579 37578 37577 37576 37575 37574 37573 37572 37571 37570 37569 37568 37567 37566 37565 37564 37563 37562 37561 37560 37559 37558 37557 37556 37555 37554 37553 37552 37551 37550 37549 37548 37547 37546 37545 37544 37543 37542 37541 37540 37539 37538 37537 37536 37535 37534 37533 37532 37531 37530 37529 37528 37527 37526 37525 37524 37523 37522 37521 37520 37519 37518 37517 37516 37515 37514 37513 37512 37511 37510 37509 37508 37507 37506 37505 37504 37503 37502 37501 37500 37499 37498 37497 37496 37495 37494 37493 37492 37491 37490 37489 37488 37487 37486 37485 37484 37483 37482 37481 37480 37479 37478 37477 37476 37475 37474 37473 37472 37471 37470 37469 37468 37467 37466 37465 37464 37463 37462 37461 37460 37459 37458 37457 37456 37455 37454 37453 37452 37451 37450 37449 37448 37447 37446 37445 37444 37443 37442 37441 37440 37439 37438 37437 37436 37435 37434 37433 37432 37431 2002..2820 37314 2821 37315 2822 37316 2823 37317 2824 37318 2825 37319 2826 37320 2827 37321 2828 37322 2829 37323 2830 37324 2831 37325 2832 37326 2833 37327 2834 37328 2835 37329 2836 37330 2837 37331 2838 37332 2839 37333 2840 37334 2841 37335 2842 37336 2843 37337 2844 37338 2845 37339 2846 37340 2847 37341 2848 37342 2849 37343 2850 37344 2851 37345 2852 37346 2853 37347 2854 37348 2855 37349 2856 37350 2857 37351 2858 37352 2859 37353 2860 37354 2861 37355 2862 37356 2863 37357 2864 37358 2865 37359 2866 37360 2867 37361 2868 37362 2869 37363 2870 37364 2871 37365 2872 37366 2873 37367 2874 37368 2875 37369 2876 37370 2877 37371 2878 37372 2879 37373 2880 37374 2881 37375 2882 37376 2883 37377 2884 37378 2885 37379 2886 37380 2887 37381 2888 37382 2889 37383 2890 37384 2891 37385 2892 37386 2893 37387 2894 37388 2895 37389 2896 37390 2897 37391 2898 37392 2899 37393 2900 37394 2901 37395 2902 37396 2903 37397 2904 37398 2905 37399 2906 37400 2907 37401 2908 37402 2909 37403 2910 37404 2911 37405 2912 37406 2913 37407 2914 37408 2915 37409 2916 37410 2917 37411 2918 37412 2919 37413 2920 37414 2921 37415 2922 37416 2923 37417 2924 37418 2925 37419 2926 37420 2927 37421 2928 37422 2929 37423 2930 37424 2931 37425 2932 37426 2933 37427 2934 37428 2935 37429 2936 37430 2937) ";
        let nodes = nodes_ids(&mut repr).unwrap();
    }


    #[test]
    fn test_edge() {
        let reprs = &[
            ("(edge 0 1 2)", Edge{id: 0, src: 1, tgt: 2}),
            ("(edge 301404 61938 61939)", Edge{id: 301404, src: 61938, tgt: 61939})
        ];
        for (repr, expect) in reprs.iter() {
            let e = edge(&mut repr.clone()).unwrap();
            assert_eq!(
                &e,
                expect
            );
        }
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
    fn test_clusters() {
        let reprs = &["(cluster 7
(nodes 2002..37313)
(edges 0..70708)
)",
"(cluster 1832
(nodes 37830 37829 37828 37827 37826 37825 37824 37823 37822 37821 37820 37819 37818 37817 37816 37815 37814 37813 37812 37811 37810 37809 37808 37807 37806 37805 37804 37803 37802 37801 37800 37799 37798 37797 37796 37795 37794 37793 37792 37791 37790 37789 37788 37787 37786 37785 37784 37783 37782 37781 37780 37779 37778 37777 37776 37775 37774 37773 37772 37771 37770 37769 37768 37767 37766 37765 37764 37763 37762 37761 37760 37759 37758 37757 37756 37755 37754 37753 37752 37751 37750 37749 37748 37747 37746 37745 37744 37743 37742 37741 37740 37739 37738 37737 37736 37735 37734 37733 37732 37731 37730 37729 37728 37727 37726 37725 37724 37723 37722 37721 37720 37719 37718 37717 37716 37715 37714 37713 37712 37711 37710 37709 37708 37707 37706 37705 37704 37703 37702 37701 37700 37699 37698 37697 37696 37695 37694 37693 37692 37691 37690 37689 37688 37687 37686 37685 37684 37683 37682 37681 37680 37679 37678 37677 37676 37675 37674 37673 37672 37671 37670 37669 37668 37667 37666 37665 37664 37663 37662 37661 37660 37659 37658 37657 37656 37655 37654 37653 37652 37651 37650 37649 37648 37647 37646 37645 37644 37643 37642 37641 37640 37639 37638 37637 37636 37635 37634 37633 37632 37631 37630 37629 37628 37627 37626 37625 37624 37623 37622 37621 37620 37619 37618 37617 37616 37615 37614 37613 37612 37611 37610 37609 37608 37607 37606 37605 37604 37603 37602 37601 37600 37599 37598 37597 37596 37595 37594 37593 37592 37591 37590 37589 37588 37587 37586 37585 37584 37583 37582 37581 37580 37579 37578 37577 37576 37575 37574 37573 37572 37571 37570 37569 37568 37567 37566 37565 37564 37563 37562 37561 37560 37559 37558 37557 37556 37555 37554 37553 37552 37551 37550 37549 37548 37547 37546 37545 37544 37543 37542 37541 37540 37539 37538 37537 37536 37535 37534 37533 37532 37531 37530 37529 37528 37527 37526 37525 37524 37523 37522 37521 37520 37519 37518 37517 37516 37515 37514 37513 37512 37511 37510 37509 37508 37507 37506 37505 37504 37503 37502 37501 37500 37499 37498 37497 37496 37495 37494 37493 37492 37491 37490 37489 37488 37487 37486 37485 37484 37483 37482 37481 37480 37479 37478 37477 37476 37475 37474 37473 37472 37471 37470 37469 37468 37467 37466 37465 37464 37463 37462 37461 37460 37459 37458 37457 37456 37455 37454 37453 37452 37451 37450 37449 37448 37447 37446 37445 37444 37443 37442 37441 37440 37439 37438 37437 37436 37435 37434 37433 37432 37431 2002..2820 37314 2821 37315 2822 37316 2823 37317 2824 37318 2825 37319 2826 37320 2827 37321 2828 37322 2829 37323 2830 37324 2831 37325 2832 37326 2833 37327 2834 37328 2835 37329 2836 37330 2837 37331 2838 37332 2839 37333 2840 37334 2841 37335 2842 37336 2843 37337 2844 37338 2845 37339 2846 37340 2847 37341 2848 37342 2849 37343 2850 37344 2851 37345 2852 37346 2853 37347 2854 37348 2855 37349 2856 37350 2857 37351 2858 37352 2859 37353 2860 37354 2861 37355 2862 37356 2863 37357 2864 37358 2865 37359 2866 37360 2867 37361 2868 37362 2869 37363 2870 37364 2871 37365 2872 37366 2873 37367 2874 37368 2875 37369 2876 37370 2877 37371 2878 37372 2879 37373 2880 37374 2881 37375 2882 37376 2883 37377 2884 37378 2885 37379 2886 37380 2887 37381 2888 37382 2889 37383 2890 37384 2891 37385 2892 37386 2893 37387 2894 37388 2895 37389 2896 37390 2897 37391 2898 37392 2899 37393 2900 37394 2901 37395 2902 37396 2903 37397 2904 37398 2905 37399 2906 37400 2907 37401 2908 37402 2909 37403 2910 37404 2911 37405 2912 37406 2913 37407 2914 37408 2915 37409 2916 37410 2917 37411 2918 37412 2919 37413 2920 37414 2921 37415 2922 37416 2923 37417 2924 37418 2925 37419 2926 37420 2927 37421 2928 37422 2929 37423 2930 37424 2931 37425 2932 37426 2933 37427 2934 37428 2935 37429 2936 37430 2937)
(edges 70709..73629)
)",

"(cluster 8
(nodes 37830 37829 37828 37827 37826 37825 37824 37823 37822 37821 37820 37819 37818 37817 37816 37815 37814 37813 37812 37811 37810 37809 37808 37807 37806 37805 37804 37803 37802 37801 37800 37799 37798 37797 37796 37795 37794 37793 37792 37791 37790 37789 37788 37787 37786 37785 37784 37783 37782 37781 37780 37779 37778 37777 37776 37775 37774 37773 37772 37771 37770 37769 37768 37767 37766 37765 37764 37763 37762 37761 37760 37759 37758 37757 37756 37755 37754 37753 37752 37751 37750 37749 37748 37747 37746 37745 37744 37743 37742 37741 37740 37739 37738 37737 37736 37735 37734 37733 37732 37731 37730 37729 37728 37727 37726 37725 37724 37723 37722 37721 37720 37719 37718 37717 37716 37715 37714 37713 37712 37711 37710 37709 37708 37707 37706 37705 37704 37703 37702 37701 37700 37699 37698 37697 37696 37695 37694 37693 37692 37691 37690 37689 37688 37687 37686 37685 37684 37683 37682 37681 37680 37679 37678 37677 37676 37675 37674 37673 37672 37671 37670 37669 37668 37667 37666 37665 37664 37663 37662 37661 37660 37659 37658 37657 37656 37655 37654 37653 37652 37651 37650 37649 37648 37647 37646 37645 37644 37643 37642 37641 37640 37639 37638 37637 37636 37635 37634 37633 37632 37631 37630 37629 37628 37627 37626 37625 37624 37623 37622 37621 37620 37619 37618 37617 37616 37615 37614 37613 37612 37611 37610 37609 37608 37607 37606 37605 37604 37603 37602 37601 37600 37599 37598 37597 37596 37595 37594 37593 37592 37591 37590 37589 37588 37587 37586 37585 37584 37583 37582 37581 37580 37579 37578 37577 37576 37575 37574 37573 37572 37571 37570 37569 37568 37567 37566 37565 37564 37563 37562 37561 37560 37559 37558 37557 37556 37555 37554 37553 37552 37551 37550 37549 37548 37547 37546 37545 37544 37543 37542 37541 37540 37539 37538 37537 37536 37535 37534 37533 37532 37531 37530 37529 37528 37527 37526 37525 37524 37523 37522 37521 37520 37519 37518 37517 37516 37515 37514 37513 37512 37511 37510 37509 37508 37507 37506 37505 37504 37503 37502 37501 37500 37499 37498 37497 37496 37495 37494 37493 37492 37491 37490 37489 37488 37487 37486 37485 37484 37483 37482 37481 37480 37479 37478 37477 37476 37475 37474 37473 37472 37471 37470 37469 37468 37467 37466 37465 37464 37463 37462 37461 37460 37459 37458 37457 37456 37455 37454 37453 37452 37451 37450 37449 37448 37447 37446 37445 37444 37443 37442 37441 37440 37439 37438 37437 37436 37435 37434 37433 37432 37431 2002..2820 37314 2821 37315 2822 37316 2823 37317 2824 37318 2825 37319 2826 37320 2827 37321 2828 37322 2829 37323 2830 37324 2831 37325 2832 37326 2833 37327 2834 37328 2835 37329 2836 37330 2837 37331 2838 37332 2839 37333 2840 37334 2841 37335 2842 37336 2843 37337 2844 37338 2845 37339 2846 37340 2847 37341 2848 37342 2849 37343 2850 37344 2851 37345 2852 37346 2853 37347 2854 37348 2855 37349 2856 37350 2857 37351 2858 37352 2859 37353 2860 37354 2861 37355 2862 37356 2863 37357 2864 37358 2865 37359 2866 37360 2867 37361 2868 37362 2869 37363 2870 37364 2871 37365 2872 37366 2873 37367 2874 37368 2875 37369 2876 37370 2877 37371 2878 37372 2879 37373 2880 37374 2881 37375 2882 37376 2883 37377 2884 37378 2885 37379 2886 37380 2887 37381 2888 37382 2889 37383 2890 37384 2891 37385 2892 37386 2893 37387 2894 37388 2895 37389 2896 37390 2897 37391 2898 37392 2899 37393 2900 37394 2901 37395 2902 37396 2903 37397 2904 37398 2905 37399 2906 37400 2907 37401 2908 37402 2909 37403 2910 37404 2911 37405 2912 37406 2913 37407 2914 37408 2915 37409 2916 37410 2917 37411 2918 37412 2919 37413 2920 37414 2921 37415 2922 37416 2923 37417 2924 37418 2925 37419 2926 37420 2927 37421 2928 37422 2929 37423 2930 37424 2931 37425 2932 37426 2933 37427 2934 37428 2935 37429 2936 37430 2937)
(edges 70709..73629)
(cluster 1832
(nodes 37830 37829 37828 37827 37826 37825 37824 37823 37822 37821 37820 37819 37818 37817 37816 37815 37814 37813 37812 37811 37810 37809 37808 37807 37806 37805 37804 37803 37802 37801 37800 37799 37798 37797 37796 37795 37794 37793 37792 37791 37790 37789 37788 37787 37786 37785 37784 37783 37782 37781 37780 37779 37778 37777 37776 37775 37774 37773 37772 37771 37770 37769 37768 37767 37766 37765 37764 37763 37762 37761 37760 37759 37758 37757 37756 37755 37754 37753 37752 37751 37750 37749 37748 37747 37746 37745 37744 37743 37742 37741 37740 37739 37738 37737 37736 37735 37734 37733 37732 37731 37730 37729 37728 37727 37726 37725 37724 37723 37722 37721 37720 37719 37718 37717 37716 37715 37714 37713 37712 37711 37710 37709 37708 37707 37706 37705 37704 37703 37702 37701 37700 37699 37698 37697 37696 37695 37694 37693 37692 37691 37690 37689 37688 37687 37686 37685 37684 37683 37682 37681 37680 37679 37678 37677 37676 37675 37674 37673 37672 37671 37670 37669 37668 37667 37666 37665 37664 37663 37662 37661 37660 37659 37658 37657 37656 37655 37654 37653 37652 37651 37650 37649 37648 37647 37646 37645 37644 37643 37642 37641 37640 37639 37638 37637 37636 37635 37634 37633 37632 37631 37630 37629 37628 37627 37626 37625 37624 37623 37622 37621 37620 37619 37618 37617 37616 37615 37614 37613 37612 37611 37610 37609 37608 37607 37606 37605 37604 37603 37602 37601 37600 37599 37598 37597 37596 37595 37594 37593 37592 37591 37590 37589 37588 37587 37586 37585 37584 37583 37582 37581 37580 37579 37578 37577 37576 37575 37574 37573 37572 37571 37570 37569 37568 37567 37566 37565 37564 37563 37562 37561 37560 37559 37558 37557 37556 37555 37554 37553 37552 37551 37550 37549 37548 37547 37546 37545 37544 37543 37542 37541 37540 37539 37538 37537 37536 37535 37534 37533 37532 37531 37530 37529 37528 37527 37526 37525 37524 37523 37522 37521 37520 37519 37518 37517 37516 37515 37514 37513 37512 37511 37510 37509 37508 37507 37506 37505 37504 37503 37502 37501 37500 37499 37498 37497 37496 37495 37494 37493 37492 37491 37490 37489 37488 37487 37486 37485 37484 37483 37482 37481 37480 37479 37478 37477 37476 37475 37474 37473 37472 37471 37470 37469 37468 37467 37466 37465 37464 37463 37462 37461 37460 37459 37458 37457 37456 37455 37454 37453 37452 37451 37450 37449 37448 37447 37446 37445 37444 37443 37442 37441 37440 37439 37438 37437 37436 37435 37434 37433 37432 37431 2002..2820 37314 2821 37315 2822 37316 2823 37317 2824 37318 2825 37319 2826 37320 2827 37321 2828 37322 2829 37323 2830 37324 2831 37325 2832 37326 2833 37327 2834 37328 2835 37329 2836 37330 2837 37331 2838 37332 2839 37333 2840 37334 2841 37335 2842 37336 2843 37337 2844 37338 2845 37339 2846 37340 2847 37341 2848 37342 2849 37343 2850 37344 2851 37345 2852 37346 2853 37347 2854 37348 2855 37349 2856 37350 2857 37351 2858 37352 2859 37353 2860 37354 2861 37355 2862 37356 2863 37357 2864 37358 2865 37359 2866 37360 2867 37361 2868 37362 2869 37363 2870 37364 2871 37365 2872 37366 2873 37367 2874 37368 2875 37369 2876 37370 2877 37371 2878 37372 2879 37373 2880 37374 2881 37375 2882 37376 2883 37377 2884 37378 2885 37379 2886 37380 2887 37381 2888 37382 2889 37383 2890 37384 2891 37385 2892 37386 2893 37387 2894 37388 2895 37389 2896 37390 2897 37391 2898 37392 2899 37393 2900 37394 2901 37395 2902 37396 2903 37397 2904 37398 2905 37399 2906 37400 2907 37401 2908 37402 2909 37403 2910 37404 2911 37405 2912 37406 2913 37407 2914 37408 2915 37409 2916 37410 2917 37411 2918 37412 2919 37413 2920 37414 2921 37415 2922 37416 2923 37417 2924 37418 2925 37419 2926 37420 2927 37421 2928 37422 2929 37423 2930 37424 2931 37425 2932 37426 2933 37427 2934 37428 2935 37429 2936 37430 2937)
(edges 70709..73629)
)
)"
    ];

        for repr in reprs.into_iter() {
            dbg!(repr);
            cluster(&mut repr.clone()).unwrap();
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