pub enum TokenModifier {
    Optional,
    OneOrMore,
    Exact(usize),
    AtLeast(usize),
    Between(usize, usize),
}
