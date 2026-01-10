import numpy as np
import os
import pygame

from quantum_animal_shogi import Environment


IMAGE_SIZE = 64


animal_images = list(map(
    lambda name: pygame.transform.scale(pygame.image.load(f"{os.path.dirname(__file__)}/res/{name}.png"), (IMAGE_SIZE, IMAGE_SIZE)),
    ["chick", "giraffe", "elephant", "lion", "chicken"]
))


def create_observation_surface(observation):
    def create_piece_surface(piece):
        result = pygame.surface.Surface((3 * IMAGE_SIZE, 3 * IMAGE_SIZE), pygame.SRCALPHA)

        pygame.draw.polygon(result, (0xff, 0xff, 0xff), [(0.0 * IMAGE_SIZE + 1, 3 * IMAGE_SIZE - 2), (3.0 * IMAGE_SIZE - 2, 3 * IMAGE_SIZE - 2), (3.0 * IMAGE_SIZE - 2, 1 * IMAGE_SIZE), (1.5 * IMAGE_SIZE, 0 * IMAGE_SIZE + 1), (0.0 * IMAGE_SIZE + 1, 1 * IMAGE_SIZE)])

        for j in range(5):
            if piece[j]:
                c, r = [(0, 2), (1, 2), (2, 2), (0, 1), (2, 1)][j]
                result.blit(animal_images[j], (c * IMAGE_SIZE, r * IMAGE_SIZE))

        pygame.draw.circle(result, (0xff, 0x7f, 0x7f) if piece[5] else (0x7f, 0xff, 0x7f), (1.5 * IMAGE_SIZE, 1.5 * IMAGE_SIZE), 0.5 * IMAGE_SIZE)

        return result

    observation = observation["observation"]

    def create_enemy_hands_surface():
        result = pygame.surface.Surface((2 * 3 * IMAGE_SIZE, 4 * 3 * IMAGE_SIZE))
        result.fill((0x7f, 0x7f, 0x7f))

        for i, piece in enumerate(filter(lambda piece: piece[8], observation[12:])):
            piece_surface = create_piece_surface(piece)
            c, r = [(1, 0), (0, 0), (1, 1), (0, 1), (1, 2), (0, 2), (1, 3), (0, 3)][i]

            result.blit(pygame.transform.flip(piece_surface, True, True), (c * 3 * IMAGE_SIZE, r * 3 * IMAGE_SIZE))

        return result

    def create_board_surface():
        result = pygame.surface.Surface((3 * 3 * IMAGE_SIZE, 4 * 3 * IMAGE_SIZE))
        result.fill((0x7f, 0x7f, 0x7f))

        for i in range(1, 2 + 1):
            pygame.draw.line(result, (0x00, 0x00, 0x00), (i * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE), (i * 3 * IMAGE_SIZE, 4 * 3 * IMAGE_SIZE))

        for i in range(1, 3 + 1):
            pygame.draw.line(result, (0x00, 0x00, 0x00), (0 * 3 * IMAGE_SIZE, i * 3 * IMAGE_SIZE), (3 * 3 * IMAGE_SIZE, i * 3 * IMAGE_SIZE))

        for i, piece in enumerate(observation[: 12]):
            if np.any(piece):
                piece_surface = create_piece_surface(piece)
                c, r = i % 3, i // 3

                result.blit(piece_surface if piece[7] else pygame.transform.flip(piece_surface, True, True), (c * 3 * IMAGE_SIZE, r * 3 * IMAGE_SIZE))

        return result

    def create_ally_hand_surface():
        result = pygame.surface.Surface((2 * 3 * IMAGE_SIZE, 4 * 3 * IMAGE_SIZE))
        result.fill((0x7f, 0x7f, 0x7f))

        for i, piece in enumerate(filter(lambda piece: piece[7], observation[12:])):
            piece_surface = create_piece_surface(piece)
            c, r = [(0, 3), (1, 3), (0, 2), (1, 2), (0, 1), (1, 1), (0, 0), (1, 0)][i]

            result.blit(piece_surface, (c * 3 * IMAGE_SIZE, r * 3 * IMAGE_SIZE))

        return result

    result = pygame.surface.Surface((9 * 3 * IMAGE_SIZE, 4 * 3 * IMAGE_SIZE))

    result.blit(create_enemy_hands_surface(), (0 * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE))
    result.blit(create_board_surface(),       (3 * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE))  # noqa: E241
    result.blit(create_ally_hand_surface(),   (7 * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE))  # noqa: E241

    return result


def get_action_item(pos):
    candidates = [
        [None, None, None,  0,  1,  2, None, 18, 19],  # noqa: E241
        [None, None, None,  3,  4,  5, None, 16, 17],  # noqa: E241
        [None, None, None,  6,  7,  8, None, 14, 15],  # noqa: E241
        [None, None, None,  9, 10, 11, None, 12, 13]   # noqa: E241
    ]

    return candidates[pos[1] // (3 * IMAGE_SIZE)][pos[0] // (3 * IMAGE_SIZE)]


def get_pos(action_item):
    candidates = [
        (0 * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE),
        (1 * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE),
        (2 * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE),
        (0 * 3 * IMAGE_SIZE, 1 * 3 * IMAGE_SIZE),
        (1 * 3 * IMAGE_SIZE, 1 * 3 * IMAGE_SIZE),
        (2 * 3 * IMAGE_SIZE, 1 * 3 * IMAGE_SIZE),
        (0 * 3 * IMAGE_SIZE, 2 * 3 * IMAGE_SIZE),
        (1 * 3 * IMAGE_SIZE, 2 * 3 * IMAGE_SIZE),
        (2 * 3 * IMAGE_SIZE, 2 * 3 * IMAGE_SIZE),
        (0 * 3 * IMAGE_SIZE, 3 * 3 * IMAGE_SIZE),
        (1 * 3 * IMAGE_SIZE, 3 * 3 * IMAGE_SIZE),
        (2 * 3 * IMAGE_SIZE, 3 * 3 * IMAGE_SIZE)
    ]

    return candidates[action_item]


def play(action_fn):
    pygame.init()
    pygame.display.set_caption("りょうしどうぶつしょうぎ")
    screen = pygame.display.set_mode((9 * 3 * IMAGE_SIZE, 4 * 3 * IMAGE_SIZE))

    def select_action(observation):
        result_0 = None

        actions_candidates = list(map(
            lambda action: (action // (3 * 4), action % (3 * 4)),
            np.where(observation["action_mask"])[0]
        ))

        board_surface = create_observation_surface(observation)

        screen.blit(board_surface, (0, 0))
        pygame.display.update()

        while True:
            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    env.close()
                    pygame.quit()
                    exit(0)

                if event.type == pygame.MOUSEBUTTONUP and event.button == 1:
                    action_item = get_action_item(event.pos)

                    if result_0 is None:
                        candidates = list(map(
                            lambda action: action[1],
                            filter(
                                lambda action: action[0] == action_item,
                                actions_candidates
                            )
                        ))

                        if not candidates:
                            continue

                        screen.blit(board_surface, (0, 0))

                        candidate_surface = pygame.Surface((3 * 3 * IMAGE_SIZE, 4 * 3 * IMAGE_SIZE), pygame.SRCALPHA)
                        for result_1_candidate in candidates:
                            pygame.draw.rect(candidate_surface, (0x00, 0x00, 0xff, 0x3f), (*get_pos(result_1_candidate), 3 * IMAGE_SIZE, 3 * IMAGE_SIZE))
                        screen.blit(candidate_surface, (3 * 3 * IMAGE_SIZE, 0 * 3 * IMAGE_SIZE))

                        pygame.display.update()

                        result_0 = action_item
                    else:
                        if action_item not in candidates:
                            screen.blit(board_surface, (0, 0))
                            pygame.display.update()

                            result_0 = None

                            continue

                        return result_0 * (4 * 3) + action_item

    env = Environment(render_mode="human")
    env.reset()

    for agent in env.agent_iter():
        observation, reward, termination, truncation, info = env.last()

        if termination or truncation:
            env.step(None)
            continue

        match agent:
            case "player_0": action = select_action(observation)
            case "player_1": action = action_fn(observation)

        env.step(action)

    env.close()
    pygame.quit()
