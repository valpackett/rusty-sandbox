use fs;

pub fn enter_sandbox<'a>(_: Box<Iterator<Item=&'a fs::Directory> + 'a>) -> bool {
    false
}
