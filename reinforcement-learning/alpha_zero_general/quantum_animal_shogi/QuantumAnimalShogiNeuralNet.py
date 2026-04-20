import numpy as np
import os
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim

from tqdm import tqdm

from ..NeuralNet import NeuralNet
from ..utils import AverageMeter


NUMBER_OF_CHANNELS = 512
DROPOUT_RATIO = 0.5

NUMBER_OF_EPOCHS = 10
BATCH_SIZE = 64


device = torch.device("cuda" if torch.cuda.is_available() else "cpu")


class NNModule(nn.Module):
    def __init__(self, game):
        super().__init__()

        self.c_conv_1 = nn.Conv2d(1 * (5 + 2 + 2) + 7 * (5 + 2 + 2 + 8), NUMBER_OF_CHANNELS, 3, stride=1, padding=1)
        self.c_conv_2 = nn.Conv2d(NUMBER_OF_CHANNELS, NUMBER_OF_CHANNELS, 3, stride=1, padding=1)
        self.c_conv_3 = nn.Conv2d(NUMBER_OF_CHANNELS, NUMBER_OF_CHANNELS, 3, stride=1, padding=1)
        self.c_conv_4 = nn.Conv2d(NUMBER_OF_CHANNELS, NUMBER_OF_CHANNELS, 3, stride=1, padding=1)

        self.c_norm_1 = nn.BatchNorm2d(NUMBER_OF_CHANNELS)
        self.c_norm_2 = nn.BatchNorm2d(NUMBER_OF_CHANNELS)
        self.c_norm_3 = nn.BatchNorm2d(NUMBER_OF_CHANNELS)
        self.c_norm_4 = nn.BatchNorm2d(NUMBER_OF_CHANNELS)

        self.fc_linear_1 = nn.Linear(NUMBER_OF_CHANNELS * 4 * 3, 1024)
        self.fc_norm_1 = nn.BatchNorm1d(1024)

        self.fc_linear_2 = nn.Linear(1024, 512)
        self.fc_norm_2 = nn.BatchNorm1d(512)

        self.p_linear = nn.Linear(512, game.getActionSize())
        self.v_linear = nn.Linear(512, 1)

    def forward(self, x):
        x = F.relu(self.c_norm_1(self.c_conv_1(x)))
        x = F.relu(self.c_norm_2(self.c_conv_2(x)))
        x = F.relu(self.c_norm_3(self.c_conv_3(x)))
        x = F.relu(self.c_norm_4(self.c_conv_4(x)))

        x = x.flatten(start_dim=1)

        x = F.dropout(self.fc_norm_1(self.fc_linear_1(x)), p=DROPOUT_RATIO, training=self.training)
        x = F.dropout(self.fc_norm_2(self.fc_linear_2(x)), p=DROPOUT_RATIO, training=self.training)

        p = F.log_softmax(self.p_linear(x), dim=1)
        v = torch.tanh(self.v_linear(x))

        return p, v


class QuantumAnimalShogiNeuralNet(NeuralNet):
    def __init__(self, game):
        super().__init__(game)

        self.game = game
        self.nn_module = NNModule(self.game)

        if device == "cuda":
            self.nn_module.cuda()

    def env_to_x(self, env):
        # 観測結果を取得します。

        observation = env.observe()["observation"]

        # 観測結果を入力に変換します。CUDAのことも考えると先にtorchのテンソルに変換した方がよいのだけど、とりあえず、NumPyで。。。

        board = observation[:4 * 3]

        hand = observation[4 * 3:]
        hand_indices = np.lexsort(hand.T)
        hand = np.ravel(np.concat([hand[hand_indices][1:], np.eye(8, dtype=np.float32)[hand_indices][1:]], axis=1))
        hand = np.broadcast_to(hand[None, :], [12, 119])

        x = np.concatenate([board, hand], axis=1)

        x = np.permute_dims(
            np.reshape(
                x,
                [4, 3, 1 * (5 + 2 + 2) + 7 * (5 + 2 + 2 + 8)]
            ),
            [2, 0, 1]
        )

        return torch.from_numpy(x)

    def predict(self, env):
        xs = torch.stack([self.env_to_x(env)])

        if device == "cuda":
            xs.cuda()

        # ニューラル・ネットワークを使用して予測します。

        self.nn_module.eval()

        with torch.no_grad():
            ps, vs = self.nn_module(xs)

        # ポリシーとバリューをリターンします。

        return torch.exp(ps).data.cpu().numpy()[0], vs.data.cpu().numpy()[0]

    def get_loss_p(self, targets, outputs):
        return -torch.sum(targets * outputs) / targets.size()[0]

    def get_loss_v(self, targets, outputs):
        return torch.sum((targets - outputs.view(-1)) ** 2) / targets.size()[0]

    def train(self, examples):
        optimizer = optim.Adam(self.nn_module.parameters())

        for _ in range(NUMBER_OF_EPOCHS):
            self.nn_module.train()

            loss_ps = AverageMeter()
            loss_vs = AverageMeter()

            batch_count = len(examples) // BATCH_SIZE
            t = tqdm(range(batch_count), desc="Training Net", ascii=True)

            for _ in t:
                xs, ps_true, vs_true = list(zip(*[examples[i] for i in np.random.randint(len(examples), size=BATCH_SIZE)]))

                xs = torch.stack([self.env_to_x(env) for env in xs])
                ps_true = torch.from_numpy(np.array(ps_true, dtype=np.float32))
                vs_true = torch.from_numpy(np.array(vs_true, dtype=np.float32))

                if device == "cuda":
                    xs, ps_true, vs_true = xs.cuda(), ps_true.cuda(), vs_true.cuda()

                ps_pred, vs_pred = self.nn_module(xs)

                loss_p = self.get_loss_p(ps_true, ps_pred)
                loss_v = self.get_loss_v(vs_true, vs_pred)

                total_loss = loss_p + loss_v

                loss_ps.update(loss_p.item(), xs.size(0))
                loss_vs.update(loss_v.item(), xs.size(0))

                t.set_postfix(Loss_p=loss_ps, Loss_v=loss_vs)

                optimizer.zero_grad()
                total_loss.backward()
                optimizer.step()

    def save_checkpoint(self, folder="checkpoint", filename="checkpoint.pth.tar"):
        path = os.path.join(folder, filename)

        if not os.path.exists(folder):
            print("Checkpoint Directory does not exist! Making directory {}".format(folder))
            os.mkdir(folder)

        torch.save({"state_dict": self.nn_module.state_dict()}, path)

    def load_checkpoint(self, folder="checkpoint", filename="checkpoint.pth.tar"):
        path = os.path.join(folder, filename)

        if not os.path.exists(path):
            raise ("No model in path {}".format(path))

        map_location = None if device == "cuda" else "cpu"
        checkpoint = torch.load(path, map_location=map_location)

        self.nn_module.load_state_dict(checkpoint["state_dict"])
