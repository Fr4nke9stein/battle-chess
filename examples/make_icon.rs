// Собирает значок ярлыка (.ico) из коня cburnett:
// cargo run --release --example make_icon -- assets/battle-chess.ico
use resvg::{tiny_skia, usvg};

fn main() {
    let out = std::env::args().nth(1).expect("путь к .ico");
    let svg = include_bytes!("../assets/piece/cburnett/wN.svg");
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default()).unwrap();
    let images: Vec<(u32, Vec<u8>)> = [16u32, 24, 32, 48, 64, 128, 256]
        .into_iter()
        .map(|size| {
            let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();
            let scale = size as f32 / tree.size().width();
            resvg::render(
                &tree,
                tiny_skia::Transform::from_scale(scale, scale),
                &mut pixmap.as_mut(),
            );
            // Картинка в формате BMP внутри ICO: заголовок, BGRA снизу вверх и пустая AND-маска.
            let mask_row = (size as usize).div_ceil(32) * 4;
            let mut bmp = Vec::new();
            bmp.extend(40u32.to_le_bytes());
            bmp.extend((size as i32).to_le_bytes());
            bmp.extend((2 * size as i32).to_le_bytes());
            bmp.extend(1u16.to_le_bytes());
            bmp.extend(32u16.to_le_bytes());
            bmp.extend(0u32.to_le_bytes());
            let image_bytes = (size * size * 4) as usize + mask_row * size as usize;
            bmp.extend((image_bytes as u32).to_le_bytes());
            bmp.extend([0u8; 16]);
            for y in (0..size).rev() {
                for x in 0..size {
                    let c = pixmap.pixel(x, y).unwrap().demultiply();
                    bmp.extend([c.blue(), c.green(), c.red(), c.alpha()]);
                }
            }
            bmp.extend(vec![0u8; mask_row * size as usize]);
            (size, bmp)
        })
        .collect();

    let mut ico = Vec::new();
    ico.extend(0u16.to_le_bytes());
    ico.extend(1u16.to_le_bytes());
    ico.extend((images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len();
    for (size, data) in &images {
        let side = if *size >= 256 { 0 } else { *size as u8 };
        ico.extend([side, side, 0, 0]);
        ico.extend(1u16.to_le_bytes());
        ico.extend(32u16.to_le_bytes());
        ico.extend((data.len() as u32).to_le_bytes());
        ico.extend((offset as u32).to_le_bytes());
        offset += data.len();
    }
    for (_, data) in &images {
        ico.extend(data);
    }
    std::fs::write(out, ico).unwrap();
}
