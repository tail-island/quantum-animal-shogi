from copy import copy

from ..Game import Game
from quantum_animal_shogi import RawEnvironment


class QuantumAnimalShogiGame(Game):
    def _get_initial_env(self):
        return RawEnvironment()

    def _get_next_env(self, env, action):
        env_prime = copy(env)
        env_prime.step(action)
        return env_prime

    def _get_action_mask(self, env):
        return env.observe()["action_mask"]

    def _get_game_ended(self, env):
        if env.won():
            return  1  # noqa: E271

        if env.lost():
            return -1

        if env.draw():
            return -0.5  # 学習がゆがみそうだけど、とりあえず無視で。

        return 0

    def getInitBoard(self):
        return self._get_initial_env()

    def getBoardSize(self):
        return (4, 3)   # とりあえず持ち駒は無視します。ここが適当でも、NeuralNetをいい感じに作れば大丈夫なはず！

    def getActionSize(self):
        return (4 * 3 + 8) * 4 * 3

    def getNextState(self, board, player, action):
        return (self._get_next_env(board, action), -player)

    def getValidMoves(self, board, player):
        return self._get_action_mask(board)

    def getGameEnded(self, board, player):
        return self._get_game_ended(board)

    def getCanonicalForm(self, board, player):
        return board

    def getSymmetries(self, board, policy):
        return [(board, policy)]  # 効率が悪くなるけど、とりあえず反転は無視で。。。

    def stringRepresentation(self, board):
        return str(board)  # 無駄に長い気がするけど、とりあえずこれで。。。
