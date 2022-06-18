/// 打印颜色
fn output() {
    use p_utils::output;
    output!("hello world");
    output!(1;2;34; 5);
    let list = [1, 2, 34, 5];
    output!("{:#?}", list);   
}

fn main() {
    output();
}
    