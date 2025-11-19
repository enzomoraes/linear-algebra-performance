#!/usr/bin/env bash
echo "Starting hardware report... ⚙️" > hardware_results.txt

echo "===============================" >> hardware_results.txt
echo "     HARDWARE BENCH REPORT     " >> hardware_results.txt
echo "===============================" >> hardware_results.txt
echo >> hardware_results.txt

echo "### CPU MODEL" >> hardware_results.txt
# Encontra a linha "Nome do modelo:" e remove a label e os espaços iniciais/finais.
lscpu | grep -i "Nome do modelo" | head -n 1 | sed 's/Nome do modelo:[ \t]*//' | tr -s ' ' | sed 's/^ //g' >> hardware_results.txt
echo >> hardware_results.txt

echo "### CPU CORES AND THREADS" >> hardware_results.txt
lscpu | grep -E "^CPU\(s\):|On-line CPU|Thread|Core\(s\)|Socket" >> hardware_results.txt
echo >> hardware_results.txt

echo "### HYBRID CPU CLUSTERS (P-cores / E-cores)" >> hardware_results.txt

# 1) Official Method (sysfs)
if ls /sys/devices/system/cpu/cpu0/topology/core_type >/dev/null 2>&1; then
    echo "Detection Method: Sysfs topology (Accurate)" >> hardware_results.txt
    echo -n "P-cores: " >> hardware_results.txt
    grep -R "0" /sys/devices/system/cpu/cpu*/topology/core_type | wc -l >> hardware_results.txt

    echo -n "E-cores: " >> hardware_results.txt
    grep -R "1" /sys/devices/system/cpu/cpu*/topology/core_type | wc -l >> hardware_results.txt

# 2) Fallback (Heuristic: Frequency-Based, but Auto-Detected)
else
    echo "Detection Method: Heuristic (Frequency based — max freq cluster = P-core)" >> hardware_results.txt

    # Captura as frequências máximas por CPU (em MHz)
    mapfile -t freqs < <(lscpu -e=CPU,MAXMHZ | awk 'NR>1 {print $2+0}')

    # Descobre a frequência mais alta do sistema
    max_freq=$(printf '%s\n' "${freqs[@]}" | sort -nr | head -1)

    # Conta P-cores (freq == max_freq)
    pcores=$(printf '%s\n' "${freqs[@]}" | awk -v m="$max_freq" '$1 == m {c++} END {print c+0}')

    # Conta E-cores (freq < max_freq)
    ecores=$(printf '%s\n' "${freqs[@]}" | awk -v m="$max_freq" '$1 <  m {c++} END {print c+0}')

    echo "Max frequency detected: ${max_freq} MHz" >> hardware_results.txt
    echo "P-cores (freq == max): $pcores" >> hardware_results.txt
    echo "E-cores (freq <  max): $ecores" >> hardware_results.txt
fi

echo >> hardware_results.txt

echo "### CACHE INFORMATION" >> hardware_results.txt
# Check if cache directory exists to avoid errors
if [ -d /sys/devices/system/cpu/cpu0/cache ]; then
    for cpu in /sys/devices/system/cpu/cpu0/cache/index*; do
        level=$(cat "$cpu/level")
        type=$(cat "$cpu/type")
        size=$(cat "$cpu/size")
        echo "L${level} ${type}: ${size}" >> hardware_results.txt
    done
else
    lscpu | grep "L[1-3] cache" >> hardware_results.txt
fi
echo >> hardware_results.txt

echo "### CPU FREQUENCY (min / max)" >> hardware_results.txt
if [ -f /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq ]; then
    max=$(cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq)
    min=$(cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq)
    echo "Max Frequency: $(echo "scale=2; $max / 1000" | bc) MHz" >> hardware_results.txt
    echo "Min Frequency: $(echo "scale=2; $min / 1000" | bc) MHz" >> hardware_results.txt
else
    # Fallback using lscpu if sysfs is restricted
    lscpu | grep "MHz" >> hardware_results.txt
fi
echo >> hardware_results.txt

echo "### MEMORY INFORMATION" >> hardware_results.txt
# Added error suppression just in case dmidecode requires sudo and it's missing
if command -v dmidecode &> /dev/null; then
    sudo dmidecode -t memory | awk '
        /Memory Device/ { is_dimm=1; print ""; print "DIMM SLOT:" }
        is_dimm && /Size:/ { print "  " $0 }
        is_dimm && /Type:/ { print "  " $0 }
        is_dimm && /Speed:/ { print "  " $0 }
        is_dimm && /Manufacturer:/ { print "  " $0 }
        /^$/ { is_dimm=0 }
    ' >> hardware_results.txt
else
    echo "dmidecode tool not found." >> hardware_results.txt
fi
echo >> hardware_results.txt

echo "### TOTAL RAM" >> hardware_results.txt
free -h | grep Mem | awk '{print "Total: " $2 " / In use now: " $3}' >> hardware_results.txt
echo >> hardware_results.txt

echo "===============================" >> hardware_results.txt
echo "        END OF REPORT          " >> hardware_results.txt
echo "===============================" >> hardware_results.txt
