# Crating mazes with Rust.

I wrote an article about my journey writing this tool on https://fau.re/blog/20160206_rust_maze.html

## Examples

### Generating a gif animation of a maze being generated using various algorithms

#### Prim's algorithm

```shell
maze -r plain -g100x100 --foreground #585858 --animation --algorithm prim prim.gif
```
It produces the following gif in 295 iterations:

![Maze generated with Prim's algorithm](https://fau.re/20160206_rust_maze/prim.gif "Maze generated with Prim's algorithm")


#### Kruskal's algorithm

```shell
maze -r plain -g100x100 --foreground #585858 --animation --algorithm kruskal kruskal.gif
```
It produces the following gif in 287 iterations:

![Maze generated with Kruskal's algorithm](https://fau.re/20160206_rust_maze/kruskal.gif "Maze generated with Kruskal's algorithm")

#### Recursive Backtracking algorithm

```shell
maze -r plain -g100x100 --foreground #585858 --animation --algorithm backtracker backtracker.gif
```
It produces the following gif in only 169 iterations:

![Maze generated with the Recursive Backtracking algorithm](https://fau.re/20160206_rust_maze/backtracker.gif "Maze generated with the Recursive Backtracking algorithm")


### Different styles

#### Plain style

```shell
maze -g630x400 --foreground #ffffff --background #000000 plain.png
```
![Plain style maze in black and white](https://fau.re/20160206_rust_maze/plain.png "Plain style maze in black and white")

#### Mosaic style

```shell
maze -g 635x400 -r mosaic mosaic.png
```
![Mosaic style maze](https://fau.re/20160206_rust_maze/mosaic.png "Mosaic style maze")

#### Invaders style

```shell
maze -g 637x399 --foreground #585858 -r invaders invaders.png
```
![Invaders style maze](https://fau.re/20160206_rust_maze/invaders.png "Invaders style maze")

### Bias

The maze generator can be biased to create mazes with a certain orientation.
Here is an example of a maze with a strong vertical bias:

```shell
maze -g 630x400 --foreground #585858 -b 0.65 vertical_bias.png
```
![Maze with a strong vertical bias](https://fau.re/20160206_rust_maze/bias.png "Maze with a strong vertical bias")

Or from a different starting point, in the center of the maze:

```shell
maze -g 630x400 --foreground #585858 -b 0.65 -o 0.5x0.5 bias_centered.png
```
![Maze with a bias starting from the center](https://fau.re/20160206_rust_maze/bias_centered.png "Maze with a bias starting from the center")

### Shading

The maze generator can also shade the walls based on their distance from the starting point.
In this example, the color goes from red to white based on the length of path from the center:

```shell
maze -g630x400 -o 0.5x0.5 --gradient length maze.png
```
![Maze with shading based on distance from the center](https://fau.re/20160206_rust_maze/shading_centered.png "Maze with shading based on distance from the center")

A different can be achieved by shading based on the distance from the solution. It produces a lava river effect:

```shell
maze -g630x400 --foreground #d70000\ #585858 --gradient solution lava_river.png
```
![Maze with shading based on distance from the solution](https://fau.re/20160206_rust_maze/lava_river.png "Maze with shading based on distance from the solution")

The impact of the Prim algorithm is clearly visible : it tends to produce mazes that go from top left to bottom right in a direct way.

With the Kruskal algorithm, the effect is more subtle:

```shell
maze -g630x400 --foreground #d70000\ #585858 --gradient solution --algorithm kruskal kruskal.png
```
![Maze with shading based on distance from the solution using Kruskal's algorithm](https://fau.re/20160206_rust_maze/kruskal_shaded.png "Maze with shading based on distance from the solution using Kruskal's algorithm")


With the backtracker algorithm:
```shell
maze -g630x400 --foreground #d70000\ #585858 --gradient solution --algorithm backtracker backtracker.png
```
![Maze with shading based on distance from the solution using the Recursive Backtracking algorithm](https://fau.re/20160206_rust_maze/backtracker_shaded.png "Maze with shading based on distance from the solution using the Recursive Backtracking algorithm")
