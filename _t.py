from quantum_animal_shogi import Environment
from pettingzoo.test import api_test


env = Environment()
env.reset()
api_test(env, num_cycles=1_000, verbose_progress=False)
