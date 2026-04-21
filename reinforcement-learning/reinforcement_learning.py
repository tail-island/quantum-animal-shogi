import numpy as np
import sys

from alpha_zero_general import MCTS, dotdict
from alpha_zero_general.quantum_animal_shogi import QuantumAnimalShogiGame, QuantumAnimalShogiNeuralNet
from quantum_animal_shogi import raw_environment_from_observation


args = dotdict({
    "numMCTSSims": 25,  # Number of games moves for MCTS to simulate.
    "cpuct": 1
})


game = QuantumAnimalShogiGame()

neural_net = QuantumAnimalShogiNeuralNet(game)
neural_net.load_checkpoint("./model", "best.pth.tar")

mcts = MCTS(game, neural_net, args)


def get_action(observation):
    return np.argmax(mcts.getActionProb(raw_environment_from_observation(observation), temp=0))


if __name__ == "__main__":
    from quantum_animal_shogi.adapter import execute

    print("*** reinforcement_learning ***", file=sys.stderr)

    execute(get_action)
