crate::define_syntax_tokens! {
    array
    pub enum CollectionOperation {
        Map => "map",
        Filter => "filter",
        Fold => "fold",
    }
}
