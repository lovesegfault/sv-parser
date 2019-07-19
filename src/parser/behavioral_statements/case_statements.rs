use crate::ast::*;
use crate::parser::*;
use nom::branch::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;

// -----------------------------------------------------------------------------

#[derive(Debug, Node)]
pub enum CaseStatement<'a> {
    Normal(CaseStatementNormal<'a>),
    Matches(CaseStatementMatches<'a>),
    Inside(CaseStatementInside<'a>),
}

#[derive(Debug, Node)]
pub struct CaseStatementNormal<'a> {
    pub nodes: (
        Option<UniquePriority<'a>>,
        CaseKeyword<'a>,
        Paren<'a, CaseExpression<'a>>,
        CaseItem<'a>,
        Vec<CaseItem<'a>>,
        Symbol<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct CaseStatementMatches<'a> {
    pub nodes: (
        Option<UniquePriority<'a>>,
        CaseKeyword<'a>,
        Paren<'a, CaseExpression<'a>>,
        Symbol<'a>,
        CasePatternItem<'a>,
        Vec<CasePatternItem<'a>>,
        Symbol<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct CaseStatementInside<'a> {
    pub nodes: (
        Option<UniquePriority<'a>>,
        Symbol<'a>,
        Paren<'a, CaseExpression<'a>>,
        Symbol<'a>,
        CaseInsideItem<'a>,
        Vec<CaseInsideItem<'a>>,
        Symbol<'a>,
    ),
}

#[derive(Debug, Node)]
pub enum CaseKeyword<'a> {
    Case(Symbol<'a>),
    Casez(Symbol<'a>),
    Casex(Symbol<'a>),
}

#[derive(Debug, Node)]
pub struct CaseExpression<'a> {
    pub nodes: (Expression<'a>,),
}

#[derive(Debug, Node)]
pub enum CaseItem<'a> {
    NonDefault(CaseItemNondefault<'a>),
    Default(CaseItemDefault<'a>),
}

#[derive(Debug, Node)]
pub struct CaseItemNondefault<'a> {
    pub nodes: (
        List<Symbol<'a>, CaseItemExpression<'a>>,
        Symbol<'a>,
        StatementOrNull<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct CaseItemDefault<'a> {
    pub nodes: (Symbol<'a>, Option<Symbol<'a>>, StatementOrNull<'a>),
}

#[derive(Debug, Node)]
pub enum CasePatternItem<'a> {
    NonDefault(CasePatternItemNondefault<'a>),
    Default(CaseItemDefault<'a>),
}

#[derive(Debug, Node)]
pub struct CasePatternItemNondefault<'a> {
    pub nodes: (
        Pattern<'a>,
        Option<(Symbol<'a>, Expression<'a>)>,
        Symbol<'a>,
        StatementOrNull<'a>,
    ),
}

#[derive(Debug, Node)]
pub enum CaseInsideItem<'a> {
    NonDefault(CaseInsideItemNondefault<'a>),
    Default(CaseItemDefault<'a>),
}

#[derive(Debug, Node)]
pub struct CaseInsideItemNondefault<'a> {
    pub nodes: (OpenRangeList<'a>, Symbol<'a>, StatementOrNull<'a>),
}

#[derive(Debug, Node)]
pub struct CaseItemExpression<'a> {
    pub nodes: (Expression<'a>,),
}

#[derive(Debug, Node)]
pub struct RandcaseStatement<'a> {
    pub nodes: (
        Symbol<'a>,
        RandcaseItem<'a>,
        Vec<RandcaseItem<'a>>,
        Symbol<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct RandcaseItem<'a> {
    pub nodes: (Expression<'a>, Symbol<'a>, StatementOrNull<'a>),
}

#[derive(Debug, Node)]
pub struct OpenRangeList<'a> {
    pub nodes: (List<Symbol<'a>, OpenValueRange<'a>>,),
}

#[derive(Debug, Node)]
pub struct OpenValueRange<'a> {
    pub nodes: (ValueRange<'a>,),
}

// -----------------------------------------------------------------------------

#[parser]
pub fn case_statement(s: Span) -> IResult<Span, CaseStatement> {
    alt((
        case_statement_normal,
        case_statement_matches,
        case_statement_inside,
    ))(s)
}

#[parser]
pub fn case_statement_normal(s: Span) -> IResult<Span, CaseStatement> {
    let (s, a) = opt(unique_priority)(s)?;
    let (s, b) = case_keyword(s)?;
    let (s, c) = paren(case_expression)(s)?;
    let (s, d) = case_item(s)?;
    let (s, e) = many0(case_item)(s)?;
    let (s, f) = keyword("endcase")(s)?;
    Ok((
        s,
        CaseStatement::Normal(CaseStatementNormal {
            nodes: (a, b, c, d, e, f),
        }),
    ))
}

#[parser]
pub fn case_statement_matches(s: Span) -> IResult<Span, CaseStatement> {
    let (s, a) = opt(unique_priority)(s)?;
    let (s, b) = case_keyword(s)?;
    let (s, c) = paren(case_expression)(s)?;
    let (s, d) = keyword("matches")(s)?;
    let (s, e) = case_pattern_item(s)?;
    let (s, f) = many0(case_pattern_item)(s)?;
    let (s, g) = keyword("endcase")(s)?;
    Ok((
        s,
        CaseStatement::Matches(CaseStatementMatches {
            nodes: (a, b, c, d, e, f, g),
        }),
    ))
}

#[parser]
pub fn case_statement_inside(s: Span) -> IResult<Span, CaseStatement> {
    let (s, a) = opt(unique_priority)(s)?;
    let (s, b) = keyword("case")(s)?;
    let (s, c) = paren(case_expression)(s)?;
    let (s, d) = keyword("inside")(s)?;
    let (s, e) = case_inside_item(s)?;
    let (s, f) = many0(case_inside_item)(s)?;
    let (s, g) = keyword("endcase")(s)?;
    Ok((
        s,
        CaseStatement::Inside(CaseStatementInside {
            nodes: (a, b, c, d, e, f, g),
        }),
    ))
}

#[parser]
pub fn case_keyword(s: Span) -> IResult<Span, CaseKeyword> {
    alt((
        map(keyword("casez"), |x| CaseKeyword::Casez(x)),
        map(keyword("casex"), |x| CaseKeyword::Casex(x)),
        map(keyword("case"), |x| CaseKeyword::Case(x)),
    ))(s)
}

#[parser]
pub fn case_expression(s: Span) -> IResult<Span, CaseExpression> {
    let (s, a) = expression(s)?;
    Ok((s, CaseExpression { nodes: (a,) }))
}

#[parser]
pub fn case_item(s: Span) -> IResult<Span, CaseItem> {
    alt((
        case_item_nondefault,
        map(case_item_default, |x| CaseItem::Default(x)),
    ))(s)
}

#[parser(MaybeRecursive)]
pub fn case_item_nondefault(s: Span) -> IResult<Span, CaseItem> {
    let (s, a) = list(symbol(","), case_item_expression)(s)?;
    let (s, b) = symbol(":")(s)?;
    let (s, c) = statement_or_null(s)?;
    Ok((
        s,
        CaseItem::NonDefault(CaseItemNondefault { nodes: (a, b, c) }),
    ))
}

#[parser]
pub fn case_item_default(s: Span) -> IResult<Span, CaseItemDefault> {
    let (s, a) = keyword("default")(s)?;
    let (s, b) = opt(symbol(":"))(s)?;
    let (s, c) = statement_or_null(s)?;
    Ok((s, CaseItemDefault { nodes: (a, b, c) }))
}

#[parser]
pub fn case_pattern_item(s: Span) -> IResult<Span, CasePatternItem> {
    alt((
        case_pattern_item_nondefault,
        map(case_item_default, |x| CasePatternItem::Default(x)),
    ))(s)
}

#[parser(MaybeRecursive)]
pub fn case_pattern_item_nondefault(s: Span) -> IResult<Span, CasePatternItem> {
    let (s, a) = pattern(s)?;
    let (s, b) = opt(pair(symbol("&&&"), expression))(s)?;
    let (s, c) = symbol(":")(s)?;
    let (s, d) = statement_or_null(s)?;
    Ok((
        s,
        CasePatternItem::NonDefault(CasePatternItemNondefault {
            nodes: (a, b, c, d),
        }),
    ))
}

#[parser]
pub fn case_inside_item(s: Span) -> IResult<Span, CaseInsideItem> {
    alt((
        case_inside_item_nondefault,
        map(case_item_default, |x| CaseInsideItem::Default(x)),
    ))(s)
}

#[parser(MaybeRecursive)]
pub fn case_inside_item_nondefault(s: Span) -> IResult<Span, CaseInsideItem> {
    let (s, a) = open_range_list(s)?;
    let (s, b) = symbol(":")(s)?;
    let (s, c) = statement_or_null(s)?;
    Ok((
        s,
        CaseInsideItem::NonDefault(CaseInsideItemNondefault { nodes: (a, b, c) }),
    ))
}

#[parser]
pub fn case_item_expression(s: Span) -> IResult<Span, CaseItemExpression> {
    let (s, a) = expression(s)?;
    Ok((s, CaseItemExpression { nodes: (a,) }))
}

#[parser]
pub fn randcase_statement(s: Span) -> IResult<Span, RandcaseStatement> {
    let (s, a) = keyword("randcase")(s)?;
    let (s, b) = randcase_item(s)?;
    let (s, c) = many0(randcase_item)(s)?;
    let (s, d) = keyword("endcase")(s)?;
    Ok((
        s,
        RandcaseStatement {
            nodes: (a, b, c, d),
        },
    ))
}

#[parser(MaybeRecursive)]
pub fn randcase_item(s: Span) -> IResult<Span, RandcaseItem> {
    let (s, a) = expression(s)?;
    let (s, b) = symbol(":")(s)?;
    let (s, c) = statement_or_null(s)?;
    Ok((s, RandcaseItem { nodes: (a, b, c) }))
}

#[parser(MaybeRecursive)]
pub fn open_range_list(s: Span) -> IResult<Span, OpenRangeList> {
    let (s, a) = list(symbol(","), open_value_range)(s)?;
    Ok((s, OpenRangeList { nodes: (a,) }))
}

#[parser]
pub fn open_value_range(s: Span) -> IResult<Span, OpenValueRange> {
    let (s, a) = value_range(s)?;
    Ok((s, OpenValueRange { nodes: (a,) }))
}
