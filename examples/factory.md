# factory
![factory](/assets/trmt_v0_3_0_factory_example.webp)

### Config
```toml
[simulation]
autoplay = true
heads = 4
rule = "R1>1,L0>1:U1>0,D0>1"
speed_ms = 5.0
trail_length = 24
color_cells = true
seed = "xeumfqni"

[display]
keycast = false
colors = ["#ff9a8b", "#ffecd2", "#a8edea", "#fed6e3"]
fade_trail_color = ""
state_based_colors = false
live_colors = false
randomize_heads = false
randomize_trails = false
head_char = ["██"]
trail_char = ["▓▓"]
cell_char = "░░"
```