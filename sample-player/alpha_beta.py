import numpy as np

from copy import copy
from itertools import starmap
from math import inf
from sys import float_info

from quantum_animal_shogi import raw_environment_from_observation


MAX_DEPTH = 5


# 盤面の評価値を取得します。

def score(board, hand):
    # 駒の価値を取得します。

    def piece_advantage(piece):
        return sum(starmap(
            lambda i, advantage: advantage if piece[i] else 0,  # インデックス0〜4は、駒の可能性の有無です（インデックス5と6は、その駒が先手由来か後手由来）。
            enumerate([1, 4, 5, 100, 10])  # 「ひよこ」と「きりん」、「ぞう」、「ライオン」、「にわとり」の駒得を、適当に決め打ってみました。
        ))

    # 自分の駒の評価値を取得します。

    ally_score = sum(map(
        piece_advantage,
        np.concatenate([board[board[:, 5 + 2 + 0] == 1], hand[hand[:, 5 + 2 + 0] == 1]], axis=0)  # 自分が所有する駒を取得します。自分が所有する駒は、インデックス7が1
    ))

    # 敵の駒の評価値を取得します。

    enemy_score = sum(map(
        piece_advantage,
        np.concatenate([board[board[:, 5 + 2 + 1] == 1], hand[hand[:, 5 + 2 + 1] == 1]], axis=0)  # 敵が所有する駒を取得します。敵が所有する駒は、インデックス8が1
    ))

    # 自分の駒の評価値ー敵の駒の評価値を評価値としてリターンします。

    return ally_score - enemy_score


# アルファ・ベータ法（Wikipediaに載っていた擬似コードそのままで、特別な工夫はなし）で手を選びます。量子どうぶつしょうぎのルール実装が面倒だったので、raw_envを再利用しています。

def alpha_beta(raw_env, depth, alpha, beta):
#   if node が終端ノード or depth = 0  # noqa: E115
#       return node の評価値           # noqa: E115

    # 負けたら（終端ノードなら）最小スコアを返します。

    if raw_env.lost():
        return -float_info.max, None
        # return -1_000 + (MAX_DEPTH - depth), None  # こちらだと、遠回りせずに勝ちにいきます。βカットの効率は落ちると思うけど。。。

    # 観測を実施します。

    observation = raw_env.observe()

    board, hand = np.split(observation["observation"], [4 * 3])
    action_mask = observation["action_mask"]

    # 指定された深さに達したら、盤面の評価値をリターンします。

    if depth == 0:
        return score(board, hand), None

#   foreach child of node

    # 実行すべき手を表す変数を用意します。

    action = None

    # 合法手でループします。

    for action_prime in np.where(action_mask)[0]:
        # 手を実行します。

        raw_env_prime = copy(raw_env)  # コピーを作って、オリジナルに副作用が発生しないようにします。
        raw_env_prime.step(action_prime)

#       α := max(α, -alphabeta(child, depth-1, -β, -α))

        # 再帰呼び出しをして、手を実行した後の盤面の評価値を取得します。

        alpha_prime = -alpha_beta(raw_env_prime, depth - 1, -beta, -alpha)[0]

        # 手を実行した後の評価値の方が良いなら、手を入れ替えます。

        if alpha_prime > alpha:
            alpha = alpha_prime
            action = action_prime

#       if α ≥ β
#           break // カット

        # ベータ・カットします。

        if alpha >= beta:
            break

#   return α
    return alpha, action


# アルファ・ベータ法でアクションを選択します。

def action(observation):
    return alpha_beta(raw_environment_from_observation(observation), MAX_DEPTH, -inf, inf)[1]


# 自己対戦させてみます。

if __name__ == "__main__":
    from play import play

    play(action)
