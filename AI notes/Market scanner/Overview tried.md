# Market scanner: don't try these

Warnings behind `Overview.md`. Read only when trying something new here.

- Don't read numbers with the general recogniser (ocrs): at any margin it dropped leading "1"s, added "0"s and a "|" ("101,000" as "01,000" or "101,0000"), wrong on roughly a third of crops. The font reader reads 102 of 102.
- Don't learn digit shapes from captures: samples labelled from one table read other tables and windows as "?" or wrong (a "0" 3.3 from its own font's samples); drawing the font needs no samples. The slot-number notes say the same.
- Don't read window titles and headers with the recogniser: the serif gold font came back as "Mk" for "Market", "Av-Day ?ce" for "Avg. Day Price"; even "Tier 4" lost its "4". Compare reference crops.
- Don't tell items apart by a name read: purchase-window names read as "Dstiha)", "Gacie's at", and "Artisan's Metallurgy: Level 1" matched Level 4.
- Don't crop only the name's first line: long names wrap, and "Destiny Crystallized ..." is cut to the same two lines for the Guardian and the Destruction Stone; the icon is what differs.
- Don't compare name crops over all pixels: two names that differ in a few digits were 0.9 apart, inside the noise of the translucent window. Over ink pixels only they are 4.6 apart and the same item 0.0.
- Don't check gem rows by reading at 0.9 similarity: "Lv. 5 Doomfire Gem" read as "L.5Doomfire Gm" on most rows and the gem was lost.
- Don't detect the last page by the first row's name: the clicked row is highlighted, so the same page read differently. Read the page indicator.
- Don't trust a dropdown pick from a fresh window: the first Tier 4 pick after Alt+Y failed twice and the T3 list was read.
- Don't wait for the 5-day hover on an item with no average ("-"): there is none, and it cost 10 s of retries.
- Don't read text with ocrs: purchase-window names came back as "Taorig:Hefr1921" (Tailoring: Hellfire [19-20]) and time left as "52m" (20h52m), wrong on most of 372 crops of a pass; PP-OCRv5 read all of them.
- Don't place the 5-day hover's rows at fixed steps: a "-" day's row is shorter, and the last value of "995.0, -, -, -, 929.0" was cut ("?2?.?").
- Don't read a Market list sorted by lowest price page by page: prices move between pages. Sort by Rank.
- Don't give PP-OCRv5 a table cell's whole width: its divider line left the strip mostly blank and the read came back empty.
- Don't give a UI step a fixed wait and one check: one pass in 29 lost the honing materials to a Market list that did not open. Wait for the screen and try the unit again from a reset.
