## error

Analysis failed

## code

invalid_request

## category

invalid_input

## message

Unknown Ghidra procedure name or address&#58; RTCCSYNC_GetRtccCount

## retryable

false

## remediation

| Key    | Value                                                                                                                                                                                                                                                                                               |
|--------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| action | Use list_procedures or search_procedures (CLI&#58; \`rea search &lt;path&gt; &lt;pattern&gt; --kind procedures --provider ghidra --json\`) to inspect the current Ghidra inventory and choose an observed name or address. An absent entry does not prove the function is absent from the artifact. |

## details.operation

procedure_pseudo_code

## details.issues

| path            | reason        | message                                                             | observed                                    |
|-----------------|---------------|---------------------------------------------------------------------|---------------------------------------------|
| \["procedure"\] | invalid_value | Unknown Ghidra procedure name or address&#58; RTCCSYNC_GetRtccCount | \{"procedure"&#58;"RTCCSYNC_GetRtccCount"\} |

## details.diagnostics.diagnostic_scope

request

## details.diagnostics.target_path

/projects/\_cache/targets/thumbv8m.main-none-eabihf/release/examples/rail_ble_tx

## details.diagnostics.target_sha256

bd479ce1b987b3e9c27d0cb025020a6ab8441d693f2eca391da2da48e935db9e

## details.diagnostics.transport

unix-socket

## details.diagnostics.provider_version

12.2

## details.diagnostics.profile_digest

61d4bf231a71bffe4077b78e0570cfda1c5141ce95c2bd43a0ba46b0888585e8

## details.diagnostics.run_id

e0acbc00-8b00-49af-b963-e525e5c7f301

## details.diagnostics.runtime_root

/tmp/rea-ghidra-QWlwgk

## details.diagnostics.endpoint_path

/tmp/rea-ghidra-QWlwgk/bridge.sock

## details.diagnostics.project_root

/tmp/rea-ghidra-QWlwgk/project

## details.diagnostics.ghidra_log_path

/tmp/rea-ghidra-QWlwgk/ghidra.log

## details.diagnostics.script_log_path

/tmp/rea-ghidra-QWlwgk/script.log

## details.diagnostics.process_id

4022560

## details.diagnostics.configured_jvm_max_heap

2G

## details.diagnostics.exit_code

null

## details.diagnostics.exit_signal

null

## details.diagnostics.stop_requested

false

## details.diagnostics.diagnostic_truncated

false

## details.diagnostics.output_incomplete

false

## details.diagnostics.failure_output.stdout

| Key            | Value |
|----------------|-------|
| text           |       |
| bytes          | 0     |
| retained_bytes | 0     |

## details.diagnostics.failure_output.stderr

| Key            | Value |
|----------------|-------|
| text           |       |
| bytes          | 0     |
| retained_bytes | 0     |

## details.diagnostics.failure_output.complete

true

## details.diagnostics.failure_output.observed_oome

\[\]

## details.diagnostics.failure_cause

| Key     | Value                                                               |
|---------|---------------------------------------------------------------------|
| name    | GhidraRemoteError                                                   |
| message | Unknown Ghidra procedure name or address&#58; RTCCSYNC_GetRtccCount |
| code    | procedure_not_found                                                 |

## details.diagnostics.failure_kind

remote

## details.diagnostics.failure_message

Unknown Ghidra procedure name or address&#58; RTCCSYNC_GetRtccCount

## details.diagnostics.remote_code

procedure_not_found

## details.diagnostics.remote_message

Unknown Ghidra procedure name or address&#58; RTCCSYNC_GetRtccCount
