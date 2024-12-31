_default:
    bat justfile -l yaml

test:
    just pytest
    just rusttest
    difft testing/pytest.mc testing/rusttest.mc

pytest:
    python3 py/assembler.py testing/test.as testing/pytest.mc
    # python3 py/better_assembler.p y testing/test.as testing/pybettertest.mc

rusttest:
    cargo run -- testing/test.as testing/rusttest.mc

@diffall:
    difft testing/2048_py.mc testing/2048_rs.mc
    difft testing/calculator_py.mc testing/calculator_rs.mc
    difft testing/connect4_py.mc testing/connect4_rs.mc
    difft testing/dvd_py.mc testing/dvd_rs.mc
    difft testing/gol_py.mc testing/gol_rs.mc
    difft testing/helloworld_py.mc testing/helloworld_rs.mc
    difft testing/maze_py.mc testing/maze_rs.mc
    difft testing/minesweeper_py.mc testing/minesweeper_rs.mc
    difft testing/tetris_py.mc testing/tetris_rs.mc


@testall:
    python3 py/assembler.py testing/programs/2048.as testing/2048_py.mc
    python3 py/assembler.py testing/programs/calculator.as testing/calculator_py.mc
    python3 py/assembler.py testing/programs/connect4.as testing/connect4_py.mc
    python3 py/assembler.py testing/programs/dvd.as testing/dvd_py.mc
    python3 py/assembler.py testing/programs/gol.as testing/gol_py.mc
    python3 py/assembler.py testing/programs/helloworld.as testing/helloworld_py.mc
    python3 py/assembler.py testing/programs/maze.as testing/maze_py.mc
    python3 py/assembler.py testing/programs/minesweeper.as testing/minesweeper_py.mc
    python3 py/assembler.py testing/programs/tetris.as testing/tetris_py.mc

    cargo run -- testing/programs/2048.as testing/2048_rs.mc
    cargo run -- testing/programs/calculator.as testing/calculator_rs.mc
    cargo run -- testing/programs/connect4.as testing/connect4_rs.mc
    cargo run -- testing/programs/dvd.as testing/dvd_rs.mc
    cargo run -- testing/programs/gol.as testing/gol_rs.mc
    cargo run -- testing/programs/helloworld.as testing/helloworld_rs.mc
    cargo run -- testing/programs/maze.as testing/maze_rs.mc
    cargo run -- testing/programs/minesweeper.as testing/minesweeper_rs.mc
    cargo run -- testing/programs/tetris.as testing/tetris_rs.mc

    just diffall
