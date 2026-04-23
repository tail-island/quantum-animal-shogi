import numpy as np


rng = np.random.default_rng(1234)


# ランダムにアクションを選択します。

def get_action(observation):
    return rng.choice(np.where(observation["action_mask"])[0])


if __name__ == "__main__":
    import sys

    from quantum_animal_shogi.adapter import execute

    print("*** uniform_random ***", file=sys.stderr)

    execute(get_action, debug=True)
