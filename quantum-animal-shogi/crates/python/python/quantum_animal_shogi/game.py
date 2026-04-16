import asyncio
import json
import numpy as np
import sys

from . import Environment


IMAGE_SIZE = 64
TIMEOUT = 30


class NumpyArrayEncoder(json.encoder.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, np.ndarray):
            return obj.tolist()

        return json.encoder.JSONEncoder.default(self, obj)


class Agent:
    def __init__(self, process, log_file):
        self.process = process
        self.log_file = log_file

    async def get_response(self, request):
        self.process.stdin.write((json.dumps(request, cls=NumpyArrayEncoder) + "\n").encode("utf-8"))
        await self.process.stdin.drain()

        return json.loads((await asyncio.wait_for(self.process.stdout.readline(), TIMEOUT)).decode("utf-8"))

    async def get_action(self, observation):
        return await self.get_response({
            "command": "get_action",
            "observation": observation
        })

    async def end_game(self):
        await self.get_response({
            "command": "end_game"
        })

        try:
            await asyncio.wait_for(self.process.communicate(), TIMEOUT)
        except asyncio.TimeoutError:
            self.process.terminate()

        self.log_file.close()


async def create_agent(command, log_file):
    return Agent(await asyncio.create_subprocess_shell(command, stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=log_file), log_file)


async def play(agent_0, agent_1):
    result = {}

    agents = {
        "agent_0": agent_0,
        "agent_1": agent_1
    }

    env = Environment(render_mode="human")
    env.reset()

    for agent in env.agent_iter():
        observation, reward, termination, truncation, info = env.last()

        if termination or truncation:
            result[agent] = reward
            env.step(None)
            continue

        env.step(await agents[agent].get_action(observation))

    env.close()

    for agent in agents.values():
        try:
            await agent.end_game()
        except Exception:
            pass

    return result


async def main(agent_0_command, agent_1_command):
    rewards = await play(
        await create_agent(agent_0_command, open("./agent-0.log", mode="w")),
        await create_agent(agent_1_command, open("./agent-1.log", mode="w"))
    )

    for _, reward in sorted(rewards.items()):
        print(f"{reward: }", file=sys.stderr)


if __name__ == "__main__":
    from argparse import ArgumentParser

    parser = ArgumentParser()
    parser.add_argument("agent_0")
    parser.add_argument("agent_1")

    args = parser.parse_args()

    asyncio.run(main(args.agent_0, args.agent_1))
