## evidence_id

ev_6a4287529452339a6a4083199a3cd7d3b0a5cb7253a703fe36adbbd2fbd7c1c9

## subject.name

rail_ble_tx

## subject.digest

| Key    | Value                                                            |
|--------|------------------------------------------------------------------|
| sha256 | bd479ce1b987b3e9c27d0cb025020a6ab8441d693f2eca391da2da48e935db9e |

## subject.local_path

/projects/\_cache/targets/thumbv8m.main-none-eabihf/release/examples/rail_ble_tx

## subject.format

elf

## subject.architecture

arm

## provider

| Key     | Value  |
|---------|--------|
| id      | ghidra |
| name    | Ghidra |
| version | 12.2   |

## analysis_profile.provider

| Key     | Value  |
|---------|--------|
| id      | ghidra |
| name    | Ghidra |
| version | 12.2   |

## analysis_profile.parameters.target_kind

executable

## analysis_profile.parameters.target_format

elf

## analysis_profile.parameters.architecture

arm

## analysis_profile.parameters.available_architectures

\["arm"\]

## analysis_profile.parameters.import_mode

ephemeral-source-immutable

## analysis_profile.parameters.annotation_policy

atomic-function-entry-metadata-v1

## analysis_profile.parameters.byte_search_boundary

contiguous-eligible-initialized-blocks-v1

## analysis_profile.parameters.load_image_observations

source-mappings-entry-context-relocation-symbol-v3

## analysis_profile.parameters.function_body_evidence

complete-inclusive-ranges-v1

## analysis_profile.parameters.function_references

complete-body-and-entry-reference-manager-v3

## analysis_profile.parameters.call_relationship_coverage

observed-call-inventory-and-discovery-v7

## analysis_profile.parameters.function_dossier_coverage

observed-facets-and-call-relationships-v1

## analysis_profile.parameters.native_value_flow_projection

accepted-runtime-budgets-v1

## analysis_profile.parameters.value_flow_effect_scope

pcode-load-store-explicit-v1

## analysis_profile.parameters.string_inventory_evidence

defined-data-coverage-v1

## analysis_profile.parameters.inventory_search_projection

provider-inventory-facts-v1

## analysis_profile.parameters.data_type_observations

bit-field-base-and-lookup-v1

## analysis_profile.parameters.location_resolution

explicit-address-exact-entry-symbol-first-v3

## analysis_profile.parameters.procedure_search

primary-and-exact-entry-symbol-names-v1

## analysis_profile.parameters.procedure_file_offset_resolution

address-first-unique-target-source-mapping-v2

## analysis_profile.parameters.instruction_flow_evidence

decoded-return-pcode-v1

## analysis_profile.parameters.instruction_status

unlisted-and-containing-code-data-v3

## analysis_profile.parameters.instruction_references

exact-address-incoming-without-synthetic-entry-v3

## analysis_profile.parameters.byte_search_coverage

separate-loaded-overlays-v1

## analysis_profile.parameters.function_boundary_observations

ghidra-function-boundary-limitations-v4

## analysis_profile.parameters.no_return_repair

returning-imports-decoded-return-v2

## analysis_profile.parameters.process_launch

inspected-jvm-launch-support-v1

## analysis_profile.parameters.jump_table_evidence

typed-case-default-blocks-v1

## analysis_profile.parameters.decompiler_jump_loads

true

## analysis_profile.parameters.loader

auto-from-header

## analysis_profile.parameters.language_id

auto-from-header

## analysis_profile.parameters.compiler_spec_id

auto-default

## analysis_profile.parameters.analyzer_preset

ghidra-default

## analysis_profile.parameters.function_coordinate_context

| Key            | Value                                                            |
|----------------|------------------------------------------------------------------|
| kind           | native-function-coordinate-context-v1                            |
| selected_image | null                                                             |
| mapping_sha256 | 55a7964dc0bcdf067484a281c658e6ca8457a49405e0ea9dbd080366ee4c3198 |

## analysis_profile.digest

61d4bf231a71bffe4077b78e0570cfda1c5141ce95c2bd43a0ba46b0888585e8

## predicate_type

rea.analysis

## operation

procedure_pseudo_code

## parameters

| Key       | Value                 |
|-----------|-----------------------|
| procedure | SYSRTC_SEQ_IRQHandler |

## raw_result

null

## normalized_result

<br>/\* WARNING&#58; Globals starting with '\_' overlap smaller symbols at the same address \*/<br><br>void SYSRTC_SEQ_IRQHandler(void)<br><br>\{<br>  \_DAT_500aa060 = \_DAT_500a8060 &amp; \_DAT_500a8064;<br>  return;<br>\}<br><br>

## confidence

observed

## authority

shipped-artifact

## environment

null

## limitations

\["The session serves operations only after Ghidra reports default auto-analysis complete; incomplete analysis does not expose partial results.","Ghidra Program checkpoint mode is disabled&#58; closing discards metadata edits; original executable bytes are unchanged. For stock ELF/PE/Mach-O on Linux/macOS, set REA_GHIDRA_CHECKPOINT_DIR to a private absolute directory before starting REA (restart MCP after changing its startup environment).","Cancelling an active Ghidra request terminates its ephemeral database and discards session annotations. The selected target remains open; the next provider query imports the original artifact again.","Default-space addresses use lowercase 0x-prefixed hexadecimal; other address spaces use &lt;percent-encoded-space&gt;&#58;0x&lt;hex&gt;.","Pseudocode is Ghidra decompiler output, not original source and not text-equivalent to Hopper output.","External functions and functions without an analyzable body return null; other decompiler failures remain explicit.","This session uses Ghidra 12.2. The bridge is verified against 12.1.4; other 12.x (&gt;=12.1) builds are accepted and the observed version is reported."\]

## relationship_coverage

null

## locations

\[\]

## evidence_links

\[\]

## caller_input_path

/projects/\_cache/targets/thumbv8m.main-none-eabihf/release/examples/rail_ble_tx
