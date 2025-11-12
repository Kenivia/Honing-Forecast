import numpy as np
import matplotlib.pyplot as plt
import json
from matplotlib.widgets import Slider

# Load data
data = np.array(json.load(open("./test_cases/brute_test_5c403ba6f102e45d.json", "r")))

n, m = data[0].shape
init_idx = 69

# Create meshgrid
x = np.arange(n)
y = np.arange(m)
X, Y = np.meshgrid(x, y, indexing='ij')

# Create figure
fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')

# Function to plot surface + minima
def plot_surface_with_minima(idx):
    Z = data[idx]

    # Find minima (could be multiple)
    min_val = np.min(Z)
    minima = np.argwhere(Z == min_val)

    # Plot main surface
    surf = ax.plot_surface(X, Y, Z, cmap='viridis')

    # Overlay red dots at minima
    ax.scatter(
        minima[:, 0],
        minima[:, 1],
        Z[minima[:, 0], minima[:, 1]],
        color='red',
        s=20,
        label='Minima'
    )

    ax.set_xlabel('a1')
    ax.set_ylabel('a2')
    ax.set_zlabel('Cost')
    ax.set_title(f"Min = {min_val:.1f}")

    return surf

# Initial plot
surf = plot_surface_with_minima(init_idx)

# Slider
ax_slider = plt.axes([0.25, 0.05, 0.5, 0.03])
slider = Slider(ax_slider, "prob", 0, len(data) - 1, valinit=init_idx, valstep=1)

# Update callback
def update(val):
    idx = int(slider.val)
    ax.clear()
    plot_surface_with_minima(idx)
    fig.canvas.draw_idle()

slider.on_changed(update)

plt.show()
