use crate::ast::*;
use crate::parser::*;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;

// -----------------------------------------------------------------------------

const KEYWORDS: &[&str] = &[
    "accept_on",
    "alias",
    "always",
    "always_comb",
    "always_ff",
    "always_latch",
    "and",
    "assert",
    "assign",
    "assume",
    "automatic",
    "before",
    "begin",
    "bind",
    "bins",
    "binsof",
    "bit",
    "break",
    "buf",
    "bufif0",
    "bufif1",
    "byte",
    "case",
    "casex",
    "casez",
    "cell",
    "chandle",
    "checker",
    "class",
    "clocking",
    "cmos",
    "config",
    "const",
    "constraint",
    "context",
    "continue",
    "cover",
    "covergroup",
    "coverpoint",
    "cross",
    "deassign",
    "default",
    "defparam",
    "design",
    "disable",
    "dist",
    "do",
    "edge",
    "else",
    "end",
    "endcase",
    "endchecker",
    "endclass",
    "endclocking",
    "endconfig",
    "endfunction",
    "endgenerate",
    "endgroup",
    "endinterface",
    "endmodule",
    "endpackage",
    "endprimitive",
    "endprogram",
    "endproperty",
    "endspecify",
    "endsequence",
    "endtable",
    "endtask",
    "enum",
    "event",
    "eventually",
    "expect",
    "export",
    "extends",
    "extern",
    "final",
    "first_match",
    "for",
    "force",
    "foreach",
    "forever",
    "fork",
    "forkjoin",
    "function",
    "generate",
    "genvar",
    "global",
    "highz0",
    "highz1",
    "if",
    "iff",
    "ifnone",
    "ignore_bins",
    "illegal_bins",
    "implements",
    "implies",
    "import",
    "incdir",
    "include",
    "initial",
    "inout",
    "input",
    "inside",
    "instance",
    "int",
    "integer",
    "interconnect",
    "interface",
    "intersect",
    "join",
    "join_any",
    "join_none",
    "large",
    "let",
    "liblist",
    "library",
    "local",
    "localparam",
    "logic",
    "longint",
    "macromodule",
    "matches",
    "medium",
    "modport",
    "module",
    "nand",
    "negedge",
    "nettype",
    "new",
    "nexttime",
    "nmos",
    "nor",
    "noshowcancelled",
    "not",
    "notif0",
    "notif1",
    "null",
    "or",
    "output",
    "package",
    "packed",
    "parameter",
    "pmos",
    "posedge",
    "primitive",
    "priority",
    "program",
    "property",
    "protected",
    "pull0",
    "pull1",
    "pulldown",
    "pullup",
    "pulsestyle_ondetect",
    "pulsestyle_onevent",
    "pure",
    "rand",
    "randc",
    "randcase",
    "randsequence",
    "rcmos",
    "real",
    "realtime",
    "ref",
    "reg",
    "reject_on",
    "release",
    "repeat",
    "restrict",
    "return",
    "rnmos",
    "rpmos",
    "rtran",
    "rtranif0",
    "rtranif1",
    "s_always",
    "s_eventually",
    "s_nexttime",
    "s_until",
    "s_until_with",
    "scalared",
    "sequence",
    "shortint",
    "shortreal",
    "showcancelled",
    "signed",
    "small",
    "soft",
    "solve",
    "specify",
    "specparam",
    "static",
    "string",
    "strong",
    "strong0",
    "strong1",
    "struct",
    "super",
    "supply0",
    "supply1",
    "sync_accept_on",
    "sync_reject_on",
    "table",
    "tagged",
    "task",
    "this",
    "throughout",
    "time",
    "timeprecision",
    "timeunit",
    "tran",
    "tranif0",
    "tranif1",
    "tri",
    "tri0",
    "tri1",
    "triand",
    "trior",
    "trireg",
    "type",
    "typedef",
    "union",
    "unique",
    "unique0",
    "unsigned",
    "until",
    "until_with",
    "untyped",
    "use",
    "uwire",
    "var",
    "vectored",
    "virtual",
    "void",
    "wait",
    "wait_order",
    "wand",
    "weak",
    "weak0",
    "weak1",
    "while",
    "wildcard",
    "wire",
    "with",
    "within",
    "wor",
    "xnor",
    "xor",
];

#[derive(Debug, Node)]
pub struct Symbol<'a> {
    pub nodes: (Span<'a>, Vec<WhiteSpace<'a>>),
}

#[derive(Debug, Node)]
pub enum WhiteSpace<'a> {
    Space(Span<'a>),
    Comment(Comment<'a>),
}

#[derive(Debug)]
pub struct Paren<'a, T: 'a> {
    pub nodes: (Symbol<'a>, T, Symbol<'a>),
}

#[derive(Debug)]
pub struct Brace<'a, T: 'a> {
    pub nodes: (Symbol<'a>, T, Symbol<'a>),
}

#[derive(Debug)]
pub struct Bracket<'a, T: 'a> {
    pub nodes: (Symbol<'a>, T, Symbol<'a>),
}

#[derive(Debug)]
pub struct ApostropheBrace<'a, T: 'a> {
    pub nodes: (Symbol<'a>, T, Symbol<'a>),
}

#[derive(Debug)]
pub struct List<T, U> {
    pub nodes: (U, Vec<(T, U)>),
}

// -----------------------------------------------------------------------------

pub fn ws<'a, O, F>(f: F) -> impl Fn(Span<'a>) -> IResult<Span<'a>, (O, Vec<WhiteSpace<'a>>)>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| {
        let (s, x) = f(s)?;
        let (s, y) = many0(white_space)(s)?;
        Ok((s, (x, y)))
    }
}

pub fn symbol<'a>(t: &'a str) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Symbol<'a>> {
    move |s: Span<'a>| {
        if cfg!(feature = "trace") {
            println!(
                "{:<64} : {:<4},{:>032x} : {}",
                format!("symbol(\"{}\")", t),
                s.offset,
                s.extra[0],
                s.fragment
            );
        }
        let (s, x) = map(ws(tag(t.clone())), |x| Symbol { nodes: x })(s)?;
        Ok((clear_recursive_flags(s), x))
    }
}

pub fn keyword<'a>(t: &'a str) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Symbol<'a>> {
    move |s: Span<'a>| {
        if cfg!(feature = "trace") {
            println!(
                "{:<64} : {:<4},{:>032x} : {}",
                format!("keyword(\"{}\")", t),
                s.offset,
                s.extra[0],
                s.fragment
            );
        }
        let (s, x) = map(ws(terminated(tag(t.clone()), peek(none_of(AZ09_)))), |x| {
            Symbol { nodes: x }
        })(s)?;
        Ok((clear_recursive_flags(s), x))
    }
}

pub fn paren<'a, O, F>(f: F) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Paren<O>>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| {
        if cfg!(feature = "trace") {
            println!(
                "{:<64} : {:<4},{:>032x} : {}",
                "paren", s.offset, s.extra[0], s.fragment
            );
        }
        let (s, a) = symbol("(")(s)?;
        let (s, b) = f(s)?;
        let (s, c) = symbol(")")(s)?;
        Ok((s, Paren { nodes: (a, b, c) }))
    }
}

pub fn bracket<'a, O, F>(f: F) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Bracket<O>>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| {
        if cfg!(feature = "trace") {
            println!(
                "{:<64} : {:<4},{:>032x} : {}",
                "bracket", s.offset, s.extra[0], s.fragment
            );
        }
        let (s, a) = symbol("[")(s)?;
        let (s, b) = f(s)?;
        let (s, c) = symbol("]")(s)?;
        Ok((s, Bracket { nodes: (a, b, c) }))
    }
}

pub fn brace<'a, O, F>(f: F) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Brace<O>>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| {
        if cfg!(feature = "trace") {
            println!(
                "{:<64} : {:<4},{:>032x} : {}",
                "brace", s.offset, s.extra[0], s.fragment
            );
        }
        let (s, a) = symbol("{")(s)?;
        let (s, b) = f(s)?;
        let (s, c) = symbol("}")(s)?;
        Ok((s, Brace { nodes: (a, b, c) }))
    }
}

pub fn apostrophe_brace<'a, O, F>(
    f: F,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, ApostropheBrace<O>>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| {
        if cfg!(feature = "trace") {
            println!(
                "{:<64} : {:<4},{:>032x} : {}",
                "apostrophe_brace", s.offset, s.extra[0], s.fragment
            );
        }
        let (s, a) = symbol("'{")(s)?;
        let (s, b) = f(s)?;
        let (s, c) = symbol("}")(s)?;
        Ok((s, ApostropheBrace { nodes: (a, b, c) }))
    }
}

pub fn list<'a, O1, O2, F, G>(f: F, g: G) -> impl Fn(Span<'a>) -> IResult<Span<'a>, List<O1, O2>>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O1>,
    G: Fn(Span<'a>) -> IResult<Span<'a>, O2>,
{
    move |s: Span<'a>| {
        let (s, a) = g(s)?;
        let mut s = s.clone();
        let mut ret = Vec::new();
        loop {
            if let Ok((t, b)) = f(s) {
                let (u, c) = g(t)?;
                s = u;
                ret.push((b, c));
            } else {
                break;
            }
        }
        Ok((s, List { nodes: (a, ret) }))
    }
}

pub fn triple<'a, O1, O2, O3, F, G, H>(
    f: F,
    g: G,
    h: H,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, (O1, O2, O3)>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O1>,
    G: Fn(Span<'a>) -> IResult<Span<'a>, O2>,
    H: Fn(Span<'a>) -> IResult<Span<'a>, O3>,
{
    move |s: Span<'a>| {
        let (s, x) = f(s)?;
        let (s, y) = g(s)?;
        let (s, z) = h(s)?;
        Ok((s, (x, y, z)))
    }
}

pub fn none<'a, O, F>(_f: F) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Option<O>>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| Ok((s, None))
}

pub fn alt_left<'a, O, F, G>(f: F, _g: G) -> impl Fn(Span<'a>) -> IResult<Span<'a>, O>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
    G: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| {
        let (s, x) = f(s)?;
        Ok((s, x))
    }
}

pub fn alt_right<'a, O, F, G>(_f: F, g: G) -> impl Fn(Span<'a>) -> IResult<Span<'a>, O>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O>,
    G: Fn(Span<'a>) -> IResult<Span<'a>, O>,
{
    move |s: Span<'a>| {
        let (s, x) = g(s)?;
        Ok((s, x))
    }
}

// -----------------------------------------------------------------------------

#[parser]
pub fn white_space(s: Span) -> IResult<Span, WhiteSpace> {
    alt((
        map(multispace1, |x| WhiteSpace::Space(x)),
        map(comment, |x| WhiteSpace::Comment(x)),
    ))(s)
}

// -----------------------------------------------------------------------------

pub fn concat<'a>(a: Span<'a>, b: Span<'a>) -> Option<Span<'a>> {
    let c = str_concat::concat(a.fragment, b.fragment);
    if let Ok(c) = c {
        Some(Span {
            offset: a.offset,
            line: a.line,
            fragment: c,
            extra: a.extra,
        })
    } else {
        None
    }
}

pub fn check_recursive_flag(s: Span, id: usize) -> bool {
    let upper = id / 128;
    let lower = id % 128;

    ((s.extra[upper] >> lower) & 1) == 1
}

pub fn set_recursive_flag(s: Span, id: usize, bit: bool) -> Span {
    let upper = id / 128;
    let lower = id % 128;

    let val = if bit { 1u128 << lower } else { 0u128 };
    let mask = !(1u128 << lower);

    let mut extra = s.extra;
    extra[upper] = (extra[upper] & mask) | val;
    Span {
        offset: s.offset,
        line: s.line,
        fragment: s.fragment,
        extra,
    }
}

pub fn clear_recursive_flags(s: Span) -> Span {
    Span {
        offset: s.offset,
        line: s.line,
        fragment: s.fragment,
        extra: [0; RECURSIVE_FLAG_WORDS],
    }
}

pub fn is_keyword(s: &Span) -> bool {
    for k in KEYWORDS {
        if &s.fragment == k {
            return true;
        }
    }
    false
}

// -----------------------------------------------------------------------------

#[cfg(test)]
macro_rules! parser_test {
    ( $x:expr, $y:expr, $z:pat ) => {
        let ret = all_consuming($x)(Span::new_extra($y, [0; RECURSIVE_FLAG_WORDS]));
        if let $z = ret {
        } else {
            assert!(false, "{:?}", ret)
        }
    };
}
