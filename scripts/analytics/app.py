# Local site showing every plot in plot.py: pnpm analytics (pulls first), or streamlit run app.py
import streamlit as st

from load import load
from plot import PLOTS

st.set_page_config(page_title="HF analytics", layout="wide")
d = st.cache_data(load)()
for name, plot in PLOTS.items():
    st.header(name)
    fig, table = plot(d)
    if fig is None:
        st.write("no data")
        continue
    st.pyplot(fig)
    if table is not None:
        st.dataframe(table, hide_index=True)
