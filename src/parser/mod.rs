macro_rules! impl_CharStream {
    (for $($t:ty),+) => {
        $(impl ICharStreamParser for $t {
            fn next_char(&self) -> char {
                self.input[self.pos..].chars().next().unwrap()
            }

            fn next_char_at(&self, offset: usize) -> char {
                self.input[(self.pos + offset)..].chars().next().unwrap()
            }

            fn eof(&self) -> bool {
                self.pos >= self.input.len()
            }

            fn consume_char(&mut self) -> Result<char, &str> {
                if self.eof() {
                    return Err("All input characters already consumed");
                }

                let mut iter = self.input[self.pos..].char_indices();
                let (_, cur_char) = iter.next().unwrap();
                let (next_post, _) = iter.next().unwrap_or((1, ' '));
                self.pos += next_post;
                return Ok(cur_char);
            }

            fn starts_with(&self, s: &str) -> bool {
                &self.input[self.pos..] == s
            }

            fn consume_while<F>(&mut self, test: F) -> String
            where
                F: Fn(char) -> bool,
            {
                let mut result = String::new();
                while !self.eof() && test(self.next_char()) {
                    result.push(self.consume_char().unwrap())
                }
                return result;
            }

            fn consume_white_space(&mut self) {
                self.consume_while(char::is_whitespace);
            }
        })*
    }
}

mod css;
mod html;

pub use css::CSSParser;
pub use html::HTMLParser;

pub trait IParser {
    type Output;
    fn new(input: &str) -> Self;
    fn parse(&mut self) -> Self::Output;
}

trait ICharStreamParser: IParser {
    fn next_char(&self) -> char;
    fn next_char_at(&self, offset: usize) -> char;
    fn starts_with(&self, s: &str) -> bool;
    fn eof(&self) -> bool;
    fn consume_char(&mut self) -> Result<char, &str>;
    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool;
    fn consume_white_space(&mut self);
}
