pub fn minify(css: &str) -> String {
    css.chars().filter(|c| !c.is_whitespace()).collect()
}
