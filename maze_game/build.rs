fn main() {
    let mut res = winres::WindowsResource::new();

    res.set_icon("assets/maze-game_icon.ico");

    res.compile().unwrap();
}