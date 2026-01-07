import numpy as np

from quantum_animal_shogi import Environment;
from pettingzoo.test import api_test


env = Environment()
env.reset()
print(env.observation_space("player_0"))
print(env.action_space("player_0"))
obs = env.observe("player_0")
print(obs["observation"].dtype)
print(obs["action_mask"].dtype)
api_test(env, num_cycles=1_000, verbose_progress=True)


rng = np.random.default_rng(1234)


env = Environment(render_mode="human")
env.reset()

for agent in env.agent_iter():
    observation, reward, termination, truncation, info = env.last()

    if termination or truncation:
        action = None
    else:
        legal_actions = np.where(observation["action_mask"])[0]
        action = rng.choice(legal_actions)

    env.step(action)

env.close()
