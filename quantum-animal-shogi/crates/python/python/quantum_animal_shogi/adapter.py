import json
import numpy as np
import sys


def execute(get_action_fn, debug=False):
    while True:
        request = json.loads(input())

        if debug:
            print(request, file=sys.stderr)

        match request["command"]:
            case "get_action":
                observation = request["observation"]
                observation["observation"] = np.array(observation["observation"], dtype=np.float32)
                observation["action_mask"] = np.array(observation["action_mask"], dtype=np.uint8)
                observation["turn"] = int(observation["turn"])

                print(json.dumps(int(get_action_fn(observation))))
                sys.stdout.flush()

            case "end_game":
                print(json.dumps("OK"))
                sys.stdout.flush()
                break
