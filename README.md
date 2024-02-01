# Mandelbrot Set Area Calculator

## Overview
This program attempts to approximate the area of the Mandelbrot set by picking random points uniformly distributed over the disk of radius 2 centered at the origin (not including the boundary), then checking if the point is within the Mandelbrot set.

## Algorithims

### Random point on disk
A random point is selected within a box with the same side length as the diameter of the disk. The point is selected by choosing a random x value and a random y value, each within the range of [-1.0, 1.0). Then, this point is multiplied by the radius of the disk, giving the point a range of [-r, r). Then, if the point is within the disk, return the point. Otherwise, go back to the start.

### Checking if a point is withing the Mandelbrot set
The point is squared then added onto the original point repeatedly for a user defined (at compile time) number of iterations. If the point ever has a magnitude greater than 2, then we assume the point is not within the set. If we reach the maximum number of iterations without exceeding this magnitude, then we assume the point is within the set.

### Area algorithm
To get the area, we only need to store 2 variables: number of points found within the Mandelbrot set and total number of points checked. To get the area, we divide the number of points within the set by the total number of points checked, then multiply by 4pi. The 4pi comes from the area of the disk were selecting random points from.

### Multithreading
Each thread runs a loop until a message is sent to the thread to exit. This loop calculates how many points selected uniformly from the disk of radius 2 are within the Mandelbrot set. It then sends this number to the main thread. The main thread uses this number to update various variables (such as points within the Mandelbrot set) containing data from previous iterations of each thread loop. This data is then combined to give an estimate of the area of the set.
