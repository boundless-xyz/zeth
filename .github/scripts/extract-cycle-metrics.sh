#!/usr/bin/env bash
set -euo pipefail

# Extract RISC Zero cycle metrics from test output
# Usage: extract-cycle-metrics.sh <input-log> <output-json>

INPUT_FILE="${1:-test-output.log}"
OUTPUT_FILE="${2:-benchmark-results.json}"

if [[ ! -f "$INPUT_FILE" ]]; then
  echo "::error::Input file not found: $INPUT_FILE"
  exit 1
fi

# Extract session metrics
TOTAL_CYCLES=$(grep -oP '\d+(?= total cycles)' "$INPUT_FILE" || echo "0")
USER_CYCLES=$(grep -oP '\d+(?= user cycles)' "$INPUT_FILE" || echo "0")

# Extract phase cycles from cycle-tracker reports
# Format: R0VM[<cycle_count>] cycle-tracker-report-{start,end}: <phase_name>
extract_phase_cycles() {
  local phase="$1"
  local start end
  start=$(grep -oP 'R0VM\[\K\d+(?=\] cycle-tracker-report-start: '"$phase"')' "$INPUT_FILE" || echo "0")
  end=$(grep -oP 'R0VM\[\K\d+(?=\] cycle-tracker-report-end: '"$phase"')' "$INPUT_FILE" || echo "0")
  echo $((end - start))
}

READ_INPUT_CYCLES=$(extract_phase_cycles "read_input")
VALIDATION_CYCLES=$(extract_phase_cycles "validation")

# Validate all metrics are numeric
for metric in TOTAL_CYCLES USER_CYCLES READ_INPUT_CYCLES VALIDATION_CYCLES; do
  if ! [[ "${!metric}" =~ ^-?[0-9]+$ ]]; then
    echo "::error::Invalid metric value for $metric: ${!metric}"
    exit 1
  fi
done

# Output for GitHub Actions
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "total_cycles=$TOTAL_CYCLES"
    echo "user_cycles=$USER_CYCLES"
    echo "read_input_cycles=$READ_INPUT_CYCLES"
    echo "validation_cycles=$VALIDATION_CYCLES"
  } >> "$GITHUB_OUTPUT"
fi

# Generate benchmark JSON
jq -n \
  --argjson total "$TOTAL_CYCLES" \
  --argjson user "$USER_CYCLES" \
  --argjson read_input "$READ_INPUT_CYCLES" \
  --argjson validation "$VALIDATION_CYCLES" \
  '[
    {name: "total_cycles", unit: "cycles", value: $total},
    {name: "user_cycles", unit: "cycles", value: $user},
    {name: "read_input_cycles", unit: "cycles", value: $read_input},
    {name: "validation_cycles", unit: "cycles", value: $validation}
  ]' > "$OUTPUT_FILE"

cat <<EOF
Cycle metrics extracted:
  total:      $TOTAL_CYCLES
  user:       $USER_CYCLES
  read_input: $READ_INPUT_CYCLES
  validation: $VALIDATION_CYCLES
EOF
