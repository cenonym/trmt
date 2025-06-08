# builders
![builders](/assets/trmt_v0_4_0_builders_example.png)

### Config
```toml
[simulation]
heads = 6
rule = "L1>1,R1>1:R1>1,R0>0"
speed_ms = 8.0
trail_length = 16
color_cells = true
seed = ""

[display]
colors = ["rgb(241, 113, 54)","#45a8e9","229"]
fade_trail_color = "#181818" # Set to bg color of your terminal for the trails to fade out 
state_based_colors = false
live_colors = false
randomize_heads = false
randomize_trails = false
head_char = ["██"]
trail_char = ["▓▓"]
cell_char = "░░"
```