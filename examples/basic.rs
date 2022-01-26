use press_btn_continue;

fn main() {
    println!("Hello world!");
    press_btn_continue::wait("Press any key to continue...").unwrap();
}
