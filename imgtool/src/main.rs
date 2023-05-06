use std::{env, path::Path};

fn main() {
    let image_path = env::args().skip(1).next().expect("no image path given");
    let path = Path::new(&image_path);
    let img = image::open(path).unwrap();
       let rotated = img.rotate90();
       rotated.save(path).unwrap();
}
