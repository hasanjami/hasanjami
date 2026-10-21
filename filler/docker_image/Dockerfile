FROM rust:1.63-buster

COPY ./maps /filler/maps
COPY ./robots /filler/robots
COPY ./game_engine /filler/game_engine

RUN chmod +x /filler/game_engine /filler/robots/* \
    && ln -s game_engine /filler/linux_game_engine \
    && ln -s robots /filler/linux_robots

WORKDIR /filler/

ENTRYPOINT ["/bin/bash"]
