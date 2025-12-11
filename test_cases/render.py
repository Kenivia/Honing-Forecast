import numpy as np
import matplotlib.pyplot as plt
import json
from matplotlib.widgets import Slider
import matplotlib.cm as cm
import matplotlib.colors as colors

# Load data from two files
# data1 = np.array(json.load(open("./test_cases/saved brute_saddle_approx_test_prob_eb189d9f0a441959.json", "r")))

data1 = np.array(json.load(open("./test_cases/brute_saddle_approx_test_prob_eb189d9f0a441959.json", "r")))
data2 = np.array(json.load(open("./test_cases/brute_arrangement_test_prob_eb189d9f0a441959.json", "r")))
# data2 = np.array(json.load(open("./test_cases/incorrect brute_saddle_approx_test_prob_eb189d9f0a441959.json", "r")))
init_idx = 0

# Create figure with two subplots side by side
fig = plt.figure(figsize=(14, 5))
ax1 = fig.add_subplot(121)
ax2 = fig.add_subplot(122)

# Function to plot surface + minima on a specific axis
def plot_surface_with_minima(data, ax, idx):
    """
    Plots Z with imshow and enables hover tooltip showing labels for the pixel under the cursor.
    - data: the dataset to plot from
    - ax: matplotlib Axes to draw on
    - idx: index into `data`
    """
    ax.clear()

    # Prepare arrays
    Z = np.array([[min(float(y[0]),1) for y in z] for z in data[idx]])
    labels = np.array([[y[1] for y in z] for z in data[idx]], dtype=object)
    
    # Find minima (could be multiple)
    val_of_interest = np.max(Z) 
    if abs(val_of_interest) > 1:
        val_of_interest = np.min(Z)
    optima = np.argwhere(Z == val_of_interest)
    
    surf = ax.imshow(Z, cmap="viridis_r" if abs(val_of_interest) > 1 else "viridis", origin="lower")

    # Single annotation object (tooltip) — start hidden
    annot = ax.annotate(
        text="", xy=(0,0), xytext=(10,10),
        textcoords="offset points",
        bbox=dict(boxstyle="round", fc="w"),
        arrowprops=dict(arrowstyle="->"),
        visible=False
    )

    # Track last displayed indices so we don't redraw unnecessarily
    last_idx = {'i': None, 'j': None}

    def on_move(event):
        # Only respond to motion over the correct axes
        if event.inaxes != ax:
            if annot.get_visible():
                annot.set_visible(False)
                fig.canvas.draw_idle()
            return

        # event.xdata/event.ydata can be None if outside data area
        if event.xdata is None or event.ydata is None:
            if annot.get_visible():
                annot.set_visible(False)
                fig.canvas.draw_idle()
            return

        # Map data coords to integer pixel indices.
        # With origin='lower', the first row of Z is at the bottom (y=0).
        j = int(np.floor(event.xdata + 0.5))  # column index
        i = int(np.floor(event.ydata + 0.5))  # row index

        # Bounds check
        nrows, ncols = Z.shape
        if i < 0 or i >= nrows or j < 0 or j >= ncols:
            if annot.get_visible():
                annot.set_visible(False)
                fig.canvas.draw_idle()
            return

        # If hovering same pixel as before, do nothing
        if last_idx['i'] == i and last_idx['j'] == j:
            return

        last_idx['i'], last_idx['j'] = i, j

        # Get label for this pixel
        label_text = str(Z[i,j]) +" "+ labels[i, j]

        # Update annotation: position it at the data coordinates (j, i)
        annot.xy = (j, i)
        annot.set_text(label_text)
        annot.set_visible(True)

        fig.canvas.draw_idle()

    # Also hide the annotation when leaving the axes
    def on_leave(event):
        if annot.get_visible():
            annot.set_visible(False)
            fig.canvas.draw_idle()
            last_idx['i'], last_idx['j'] = None, None

    cid_move = fig.canvas.mpl_connect("motion_notify_event", on_move)
    cid_leave = fig.canvas.mpl_connect("axes_leave_event", on_leave)

    # Scatter plot for optima
    ax.scatter(
        optima[:, 1],
        optima[:, 0],
        color='red',
        s=40,
        label='Optima'
    )

    ax.set_xlabel('a1')
    ax.set_ylabel('a2')
    if val_of_interest > 1:
        ax.set_title(f"Max = {np.max(Z):.0f} Min = {val_of_interest:.0f}")
    else:
        ax.set_title(f"Max = {val_of_interest*100:.4f}% Min = {np.min(Z)*100:.4f}%")

    return surf


# Initial plot
plot_surface_with_minima(data1, ax1, init_idx)
plot_surface_with_minima(data2, ax2, init_idx)

# Slider
ax_slider = plt.axes([0.25, 0.05, 0.5, 0.03])
slider = Slider(ax_slider, "", 0, len(data1) - 1, valinit=init_idx, valstep=1)

# Update callback
def update(val):
    idx = int(slider.val)
    plot_surface_with_minima(data1, ax1, idx)
    plot_surface_with_minima(data2, ax2, idx)
    fig.canvas.draw_idle()

slider.on_changed(update)

plt.show()
