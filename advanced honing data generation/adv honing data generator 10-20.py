
import random
# fast_sim.py
import math
import os
from collections import Counter
from concurrent.futures import ProcessPoolExecutor, as_completed
import random
import numpy as np

# Precomputed thresholds for the ball ancestor distribution (original: [0.15,0.35,0.15,0.35])
BALL_THRESH = (0.15, 0.50, 0.65)  # cumulative thresholds -> 1,2,3,4

# Precomputed cumulative thresholds for roll_tap combos:
# base weights = [0.8, 0.15, 0.05]
# adjustments applied exactly as original code:
# (juice=False, scroll=False), (juice=True, scroll=False), (juice=False, scroll=True), (juice=True, scroll=True)
ROLL_THRESH = {
    (False, False): (0.8, 0.95),   # cumsum: [0.8, 0.95, 1.0]
    (True, False):  (0.5, 0.8),    # [0.5, 0.8, 1.0]
    (False, True):  (0.3, 0.75),   # [0.3, 0.75, 1.0]
    (True, True):   (0.0, 0.6),    # [0.0, 0.6, 1.0]  (first weight 0)
}

def _single_sim(rng_random):
    """
    Run a single sim using rng_random (callable returning a float in [0,1)).
    Returns (cost, juice_count) tuple.
    """
    cur_xp = 0
    cur_balls = 0
    cost = 0
    juice_count = 0
    next_free = False

    while cur_xp < 1000:
        if not next_free:
            cost += 1
        else:
            next_free = False

        if cur_balls >= 6:
            # ancestor tap sequence
            cur_balls = 0
            juice_count += 1

            # choose roll_tap with juice/scroll depending on remaining XP
            if 1000 - cur_xp <= 30:
                # Use (juice=False, scroll=False)
                t1, t2 = ROLL_THRESH[(False, False)]
            else:
                # Use (juice=True, scroll=True)
                t1, t2 = ROLL_THRESH[(True, True)]

            r = rng_random()
            if r < t1:
                rolled_xp = 10
            elif r < t2:
                rolled_xp = 20
            else:
                rolled_xp = 40

            # ancestor grace roll
            r2 = rng_random()
            if r2 < BALL_THRESH[0]:
                # case 1
                rolled_xp *= 5
            elif r2 < BALL_THRESH[1]:
                # case 2
                rolled_xp *= 3
            elif r2 < BALL_THRESH[2]:
                # case 3
                rolled_xp += 30
                cur_balls = 6
            else:
                # case 4
                rolled_xp += 10
                next_free = True

            cur_xp += rolled_xp

        else:
            # normal tap (juice=False, scroll=False)
            t1, t2 = ROLL_THRESH[(False, False)]
            r = rng_random()
            if r < t1:
                v = 10
            elif r < t2:
                v = 20
            else:
                v = 40
            cur_xp += v
            cur_balls += 2

    return cost, juice_count



def _worker_run(batch_size, seed=None):
    """
    Runs `batch_size` simulations in this worker process.
    Returns a Counter mapping (cost, juice_count) -> count
    """
    # Use Python's random.Random per process (fast C implementation)
    if seed is None:
        rng = random.Random()
    else:
        rng = random.Random(seed)

    rand = rng.random  # local ref for speed
    c = Counter()
    for _ in range(batch_size):
        tup = _single_sim(rand)
        c[tup] += 1
    return c


def run_simulations(trial_count=1000, workers=None, seed=None):
    """
    Run the Monte Carlo and return `out` dict in the same format as your original code:
    keys = str((cost, juice_count)), values = counts.
    - trial_count : total number of sims to run
    - workers : number of parallel worker processes (defaults to os.cpu_count())
    - seed : optional integer seed for reproducibility; different child seeds are derived
    """
    if workers is None:
        workers = min(15, (os.cpu_count() or 1))  # cap a bit

    # Split trials roughly evenly per worker
    base = trial_count // workers
    remainder = trial_count % workers
    batch_sizes = [base + (1 if i < remainder else 0) for i in range(workers)]
    # drop zero-sized batches (if trial_count < workers)
    batch_sizes = [b for b in batch_sizes if b > 0]

    # Create distinct seeds if seed provided
    seeds = None
    if seed is not None and len(batch_sizes) > 0:
        ss = np.random.SeedSequence(seed)
        child_seeds = ss.spawn(len(batch_sizes))
        seeds = [int(s.pool[0]) for s in child_seeds]
        # Fallback simpler seeding if above not available:
        # but numpy SeedSequence provides spawn; above line extracts integer-ish seed.
    else:
        seeds = [None] * len(batch_sizes)

    counters = []
    with ProcessPoolExecutor(max_workers=len(batch_sizes)) as ex:
        futures = []
        for b, s in zip(batch_sizes, seeds):
            futures.append(ex.submit(_worker_run, b, s))
        for fut in as_completed(futures):
            counters.append(fut.result())

    # merge counters
    total_counter = Counter()
    for c in counters:
        total_counter.update(c)

    # produce `out` with same key format as your original code (stringified tuple)
    out = {k: v for k, v in total_counter.items()}
    actual_out = {}
    for i in out:
        cost, juice = i
        if cost not in actual_out:
            actual_out[cost] = [juice*out[i], out[i]]
        else:
            actual_out[cost][0] += juice*out[i]
            actual_out[cost][1] += out[i]
    for i in actual_out:
        actual_out[i][0] /= actual_out[i][1]


    return actual_out


if __name__ == '__main__':
    import ast
    import re

    def dict_to_tsv(d, sort_by='count_desc'):
        """
        Convert dict `d` (keys like "(cost, juice_count)" or tuples) into TSV string.
        sort_by: 'count_desc' (default), 'cost_asc'
        """
        rows = []
        for v,k  in d.items():
            # parse key into a tuple (cost, juice_count)
            if isinstance(k, tuple):
                tup = k
            elif isinstance(k, str):
                try:
                    tup = ast.literal_eval(k)   # safest for strings like "(10, 2)"
                except Exception:
                    nums = re.findall(r'-?\d+', k)
                    tup = tuple(int(n) for n in nums[:2]) if nums else (None, None)
            else:
                try:
                    tup = tuple(k)
                except Exception:
                    tup = (None, None)

            # make sure we have two integer columns (fall back if missing)
            try:
                juice = float(tup[0]) if tup and tup[0] is not None else ''
            except Exception:
                juice = ''
            try:
                frequency = int(tup[1]) if tup and len(tup) > 1 and tup[1] is not None else ''
            except Exception:
                frequency = ''

            tap_count = int(v)
            rows.append((tap_count, round(juice,3), frequency, ))

        if sort_by == 'count_desc':
            rows.sort(key=lambda r: r[1], reverse=True)
        elif sort_by == 'cost_asc':
            rows.sort(key=lambda r:r[0])

        lines = ["Taps\tJuice\tFrequency"]
        lines += [f'{a}\t{b}\t{c}' for (a, b, c) in rows]
        return '\n'.join(lines)


    # Example usage:
    # assume `out` is the dict produced by your sim code
    
    # small example run
    TRIAL_COUNT = 1000*10*1000  # change as needed
    out = run_simulations(TRIAL_COUNT, workers=None, seed=12345)
    

    # show summary
    total = sum([i[1] for i in out.values()])
    print(f"Ran {total} trials. Unique outcomes: {len(out)}")
    # print a few entries sorted by frequency
    items = sorted(out.items(), key=lambda kv: kv[0])
    tsv = dict_to_tsv(out, sort_by='cost_asc')
    for k, v in items:
        print([k, round(v[0]*1000),v[1]])


    import pickle
    import json
    import time
    with open("10 or 20 adv honing data " +str(time.time()) + ".txt","w") as file:
        # dump_json = json.dumps(out)
        file.write(tsv)
    # pickle.dump(out,open("10 or 20 adv honing data " +str(time.time()) + ".pkl" , "wb"))

