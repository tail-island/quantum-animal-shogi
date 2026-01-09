from copy import copy
from functools import lru_cache
from gymnasium.spaces import Box, Dict, Discrete, MultiBinary
from pettingzoo import AECEnv

from .quantum_animal_shogi import RawEnvironment, _raw_environment_from_observation


# PettingZooの環境です。

class Environment(AECEnv):
    metadata = {"render_modes": ["human"], "name": "quantum-animal-shogi"}

    def __init__(self, render_mode=None):
        self.raw_env = RawEnvironment()  # メモリ効率を良くしたい場合は、本コードを参考にRawEnvironmentの使用を検討してください。呼び出しが変わるので、面倒だけど……。

        self.render_mode = render_mode
        self.possible_agents = ["player_0", "player_1"]

    @lru_cache(maxsize=None)
    def action_space(self, agent):
        return Discrete((4 * 3 + 8) * (4 * 3))

    def close(self):
        pass

    @lru_cache(maxsize=None)
    def observation_space(self, agent):
        return Dict({"observation": Box(low=0, high=1, shape=[4 * 3 + 8, 5 + 2 + 2]), "action_mask": MultiBinary((4 * 3 + 8) * (4 * 3)), "turn": Discrete(1_000)})

    def observe(self, agent):
        return self.observations[agent]

    def render(self):
        print(self.raw_env)
        print()

    def reset(self, seed=None, options=None):
        self.raw_env.reset()

        self.agents = copy(self.possible_agents)
        self.agent_selection = self.agents[0]

        self.observations = dict([
            (self.agents[0], self.raw_env.observe()),
            (self.agents[1], None)
        ])
        self.rewards = dict(map(lambda agent: (agent, 0), self.agents))
        self._cumulative_rewards = dict(map(lambda agent: (agent, 0), self.agents))
        self.terminations = dict(map(lambda agent: (agent, False), self.agents))
        self.truncations = dict(map(lambda agent: (agent, False), self.agents))
        self.infos = dict(map(lambda agent: (agent, {}), self.agents))

    def step(self, action):
        if self.terminations[self.agent_selection] or self.truncations[self.agent_selection]:
            self._was_dead_step(action)
            return

        reward = self.raw_env.step(action)

        if reward != 0:  # 勝敗が決定した場合です。
            self.observations[self.agents[(self.agents.index(self.agent_selection) + 0) % 2]] = self.raw_env.observe_turned()

            self.rewards[self.agents[(self.agents.index(self.agent_selection) + 0) % 2]] =  reward  # noqa: E222
            self.rewards[self.agents[(self.agents.index(self.agent_selection) + 1) % 2]] = -reward

            self.terminations[self.agents[(self.agents.index(self.agent_selection) + 0) % 2]] = True
            self.terminations[self.agents[(self.agents.index(self.agent_selection) + 1) % 2]] = True

        self.observations[self.agents[(self.agents.index(self.agent_selection) + 1) % 2]] = self.raw_env.observe()

        self._accumulate_rewards()
        self.agent_selection = self.agents[(self.agents.index(self.agent_selection) + 1) % 2]

        if self.render_mode == "human":
            self.render()


def raw_environment_from_observation(observation):
    return _raw_environment_from_observation(observation["observation"].T, observation["turn"])


__all__ = [
    "Environment",
    "raw_environment_from_observation"
]
