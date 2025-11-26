#!/bin/bash

# --- Configurações ---
N_WARMUP=10
N_REPETITIONS=50
SIZE="1000"
BLOCK_SIZE="264"
OUTPUT_FILE="test_results-$N_REPETITIONS-$SIZE-$BLOCK_SIZE.txt"

> $OUTPUT_FILE

# PERF_EVENTS="task-clock,context-switches,cpu-migrations,page-faults,instructions,cycles,cache-references,cache-misses,branches,branch-misses"
# PERF_EVENTS="task-clock,context-switches,cpu-migrations,cache-references,cache-misses,branch-misses"
# PERF_EVENTS="task-clock,cache-misses,branch-misses,cycles,instructions"
PERF_EVENTS="task-clock,cache-misses,branch-misses,instructions"


PROGRAMS=(
    "./target/release/naive_fragmented|SIZE=\"$SIZE\"|naive_fragmented"
    "./target/release/contiguous_strided|SIZE=\"$SIZE\"|contiguous_strided"
    "./target/release/contiguous_parallel_strided|SIZE=\"$SIZE\"|contiguous_parallel_strided"
    "./target/release/contiguous_tiled|SIZE=\"$SIZE\" BLOCK_SIZE=\"$BLOCK_SIZE\"|contiguous_tiled"
    "./target/release/contiguous_parallel_tiled|SIZE=\"$SIZE\" BLOCK_SIZE=\"$BLOCK_SIZE\"|contiguous_parallel_tiled"
)

detect_cores() {
    P_CORES=()
    E_CORES=()

    declare -A freq_map

    # Coleta freq máxima dos cores disponíveis
    for cpu in /sys/devices/system/cpu/cpu[0-9]*; do
        id=$(basename "$cpu" | sed 's/cpu//')

        if [[ -f "$cpu/cpufreq/cpuinfo_max_freq" ]]; then
            freq=$(cat "$cpu/cpufreq/cpuinfo_max_freq")
        else
            continue
        fi

        freq_map["$id"]=$freq
    done

    # Maior frequência → P-cores
    max_freq=$(printf '%s\n' "${freq_map[@]}" | sort -nr | head -1)

    for id in "${!freq_map[@]}"; do
        if [[ "${freq_map[$id]}" == "$max_freq" ]]; then
            P_CORES+=("$id")
        else
            E_CORES+=("$id")
        fi
    done

    P_CORES_CSV=$(IFS=, ; echo "${P_CORES[*]}")
    E_CORES_CSV=$(IFS=, ; echo "${E_CORES[*]}")

    echo "P-cores detectados: $P_CORES_CSV" >> $OUTPUT_FILE
    echo "E-cores detectados: $E_CORES_CSV" >> $OUTPUT_FILE

    # Comando para usar apenas os P-cores
    TASKSET_CMD="taskset -c $P_CORES_CSV"
    echo "usando apenas os P-cores: $P_CORES_CSV" >> $OUTPUT_FILE
}

detect_cores

for entry in "${PROGRAMS[@]}"; do
    IFS='|' read -r EXECUTABLE ENV_VARS TEST_NAME <<< "$entry"

    echo "--- Teste $TEST_NAME ---"

    echo "Warm-up... ♨️"
    for i in $(seq 1 $N_WARMUP); do
        eval "$ENV_VARS $TASKSET_CMD $EXECUTABLE > /dev/null 2>&1"
    done
    echo "Warm-up completed... ✅"

    echo "# TEST_NAME: $TEST_NAME" >> $OUTPUT_FILE
    echo "# REPEAT: $N_REPETITIONS" >> $OUTPUT_FILE

    echo "Running test... 🚀"
    eval "$ENV_VARS $TASKSET_CMD perf stat -r $N_REPETITIONS -e $PERF_EVENTS $EXECUTABLE >> $OUTPUT_FILE 2>&1"
    echo "Test completed... ✅"

    echo "" >> $OUTPUT_FILE
done

echo "--- Concluído ---"
