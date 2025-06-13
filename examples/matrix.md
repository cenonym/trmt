# matrix
Inspired by [cmatrix](https://github.com/abishekvashok/cmatrix) and [unimatrix](https://github.com/will8211/unimatrix)
![factory](/assets/trmt_v0_4_0_matrix_example.webp)

### Config
```toml
[simulation]
autoplay = true
heads = 128
rule = "S"
speed_ms = 100.0
trail_length = 16
color_cells = false
seed = ""

[display]
keycast = false
colors = ["#00ff41", "#008f11", "#004400", "#002200"]
fade_trail_color = "#181818" # Set to bg color of your terminal for the trails to fade out
state_based_colors = false
live_colors = false
randomize_heads = true
randomize_trails = true
head_char = ["ﾊ", "ﾐ", "ﾋ", "ｰ", "ｳ", "ｼ", "ﾅ", "ﾓ", "ﾆ", "ｻ"]
trail_char = ["ｦ","ｧ","ｨ","ｩ","ｪ","ｫ","ｬ","ｭ","ｮ","ｯ","ｰ","ｱ","ｲ","ｳ","ｴ","ｵ","ｶ","ｷ","ｸ","ｹ","ｺ","ｻ","ｼ","･","･"]
cell_char = " "
```