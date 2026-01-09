import numpy as np

from quantum_animal_shogi import Environment, raw_environment_from_observation
from pettingzoo.test import api_test


# PettingZooのzpi_testを実行します。

env = Environment()
env.reset()
api_test(env, num_cycles=10_000, verbose_progress=False)


# 実際にゲームをプレイしてみます。

rng = np.random.default_rng(1234)

env = Environment(render_mode="human")
env.reset()

for agent in env.agent_iter():
    observation, reward, termination, truncation, info = env.last()

    raw_env = raw_environment_from_observation(observation)

    board, hand = np.split(observation["observation"], [4 * 3])
    action_mask = observation["action_mask"]

    print(np.reshape(board, [4, 3, 5 + 2 + 2]))
    print()
    print(np.reshape(hand, [8, 5 + 2 + 2]))
    print()
    print(list(map(
        lambda legal_action: tuple(map(int, [legal_action // (4 * 3), legal_action % (4 * 3)])),
        np.where(action_mask)[0])
    ))
    print()
    print(f"reward = {reward}")
    print()

    env.step(rng.choice(np.where(observation["action_mask"])[0]) if not termination and not truncation else None)

env.close()
