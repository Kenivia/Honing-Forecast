import numpy as np
import matplotlib.pyplot as plt
import json
from matplotlib.widgets import Slider
import matplotlib.cm as cm
import matplotlib.colors as colors
# Load data
data = np.array(json.load(open("./test_cases/brute_test_66c222775b22b9be.json", "r")))

n, m = data[0].shape
init_idx = 0

# Create meshgrid
x = np.arange(n)
y = np.arange(m)
X, Y = np.meshgrid(x, y, indexing='ij')

# Create figure
fig = plt.figure()
ax = fig.add_subplot(111, )

# Function to plot surface + minima
def plot_surface_with_minima(idx):
    Z = data[idx]

    # Find minima (could be multiple)
    max_val = np.max(Z)
    minima = np.argwhere(Z == max_val)

    # Plot main surface
    # surf = ax.bar3d(X.ravel(), Y.ravel(),0,1 , 1 , 0,color=colormap_from_Z(Z.ravel()), shade=True)
    surf = ax.imshow(Z,cmap="viridis",origin="lower",vmax = max_val,vmin=min(np.min(Z), max_val-0.02))
    # surf = ax.scatter(X, Y, c=Z, cmap="viridis", marker=",", s=40)
    # surf = ax.scatter(X, Y, Z, s=3)
    # surf = ax.plot_surface(X,Y,Z,rstride=1, cstride=1,cmap="viridis")

    # Overlay red dots at minima
    ax.scatter(
       minima[:, 1],
        minima[:, 0],
         
        color='red',
        s=40,
        label='Maxima'
    )

    ax.set_xlabel('a1')
    ax.set_ylabel('a2')
    ax.set_title(f"Max = {max_val*100:.4f}% Min = { np.min(Z)*100:.4f}%, Color gradient range {max_val*100:.4f}% to {min(np.min(Z), max_val-0.02)*100:.4f}%")

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
