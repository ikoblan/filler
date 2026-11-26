#!/bin/bash

cd "$(dirname "$0")/.." || exit 1

ENGINE="${ENGINE:-./linux_game_engine}"
BOT="${BOT:-solution/filler_bot/target/release/filler_bot}"
ROBOTS="bender h2_d2 wall_e terminator"
MAPS="map00 map01 map02"
GAMES="${GAMES:-3}"

play() {
    local side=$1 robot=$2 map=$3 seed=$4
    local args=(-f "maps/$map" -q)
    [ -n "$seed" ] && args+=(-s "$seed")
    if [ "$side" = 1 ]; then
        args+=(-p1 "$BOT" -p2 "linux_robots/$robot")
    else
        args+=(-p1 "linux_robots/$robot" -p2 "$BOT")
    fi

    local out s1 s2 winner
    out=$("$ENGINE" "${args[@]}" 2>&1)
    s1=$(echo "$out" | sed -n 's/^Player1 .*: \([0-9]*\) *$/\1/p' | tail -1)
    s2=$(echo "$out" | sed -n 's/^Player2 .*: \([0-9]*\) *$/\1/p' | tail -1)
    winner=$(echo "$out" | grep -o 'Player[12] won' | tail -1 | cut -c1-7)

    local mine theirs
    if [ "$side" = 1 ]; then mine=$s1; theirs=$s2; else mine=$s2; theirs=$s1; fi

    if [ "$winner" = "Player$side" ]; then
        printf "  WIN   бот(p%s) %s : %s %-10s %s seed=%s\n" "$side" "$mine" "$theirs" "$robot" "$map" "${seed:-random}"
        return 0
    else
        printf "  LOSE  бот(p%s) %s : %s %-10s %s seed=%s\n" "$side" "${mine:-?}" "${theirs:-?}" "$robot" "$map" "${seed:-random}"
        [ -z "$s1" ] && echo "$out" | tail -5 | sed 's/^/        /'
        return 1
    fi
}

check() {
    local robot=$1 maps=${2:-$MAPS} wins=0 total=0 map side g
    echo "== $robot =="
    for map in $maps; do
        for side in 1 2; do
            for g in $(seq 1 "$GAMES"); do
                play "$side" "$robot" "$map" "$((g * 1000 + side))" && wins=$((wins + 1))
                total=$((total + 1))
            done
        done
    done
    echo "  итого: $wins из $total"
}

build() {
    (cd solution/filler_bot && cargo build --release) || exit 1
}

usage() {
    cat <<EOF
Использование: ./solution/run.sh <команда> [карта]

  build                   собрать бота (cargo build --release)
  test                    запустить юнит-тесты (cargo test)
  bender [карта]          проверка против bender
  h2_d2 [карта]           проверка против h2_d2
  wall_e [карта]          проверка против wall_e
  terminator [карта]      проверка против terminator (побеждать не обязательно)
  all                     против всех, кроме terminator, на всех картах
  full                    против всех, включая terminator
  one <робот> <карта> [seed]   одна партия бота за игрока 1, с выводом
  watch <робот> <карта> [seed] одна партия с полным выводом ходов

Примеры:
  ./solution/run.sh build
  ./solution/run.sh bender
  ./solution/run.sh wall_e map01
  GAMES=10 ./solution/run.sh all     # по 10 партий на пару
EOF
}

cmd=$1
shift
case "$cmd" in
    build) build ;;
    test) (cd solution/filler_bot && cargo test) ;;
    bender|h2_d2|wall_e|terminator) check "$cmd" "$1" ;;
    all) for r in bender h2_d2 wall_e; do check "$r"; done ;;
    full) for r in $ROBOTS; do check "$r"; done ;;
    one) play 1 "${1:-bender}" "${2:-map00}" "$3" ;;
    watch) "$ENGINE" -f "maps/${2:-map00}" -p1 "$BOT" -p2 "linux_robots/${1:-bender}" ${3:+-s "$3"} ;;
    *) usage ;;
esac