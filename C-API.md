## Types

### `Result`
A 32 bit signed integer, either a function-specific success value or one of of the following failure values:

Name                    | Value          | Description
------------------------|----------------|-------------
`NullPointer`           | `-0x0000_0001` | A pointer was null
`PointerMisaligned`     | `-0x0000_0002` | A pointer did not have the required alignment for its type
`InvalidArgument`       | `-0x0000_0003` | An argument was invalid
`ArgumentOutOfRange`    | `-0x0000_0004` | An argument was outside of its valid range
`Utf8Encoding`          | `-0x0000_0005` | A string did not have valid UTF8 encoding
`Io`                    | `-0x0000_0006` | An IO error occurred
`InvalidOperation`      | `-0x0000_0007` | The operation was invalid
`ResourceLimitReached`  | `-0x0001_0001` | The simulation reached its resource limit
`WireWidthMismatch`     | `-0x0001_0002` | The width of two or more wires mismatched
`WireWidthIncompatible` | `-0x0001_0003` | The width of a wire is incompatible with a component
`OffsetOutOfRange`      | `-0x0001_0004` | An offset was out of its valid range
`TooFewInputs`          | `-0x0001_0005` | Too few inputs were given for a component
`InvalidInputCount`     | `-0x0001_0006` | An invalid number of inputs was given for a component
`InvalidComponentType`  | `-0x0001_0007` | The type of a component was invalid for an operation
`Conflict`              | `-0x0002_0001` | The simulation caused a driver conflict
`InvalidWireId`         | `-0x0002_0002` | A wire id was not a valid wire in the simulation
`InvalidComponentId`    | `-0x0002_0003` | A component id was not a valid component in the simulation
`MalformedFormat`       | `-0x0003_0001` | The format of some data was invalid
`Unsupported`           | `-0x0003_0002` | An opperation was not supported

### `WireId`
A 32 bit unsigned integer uniquely identifying a wire in a simulation.  
A value of `0xFFFFFFFF` corresponds to the invalid ID. This ID will never be returned by the API.

### `ComponentId`
A 32 bit unsigned integer uniquely identifying a component in a simulation.  
A value of `0xFFFFFFFF` corresponds to the invalid ID. This ID will never be returned by the API.

### `LogicState`
An opaque type representing a logic state of up to 255 bits.  
Use only behind a pointer.

### `PortList`
```
struct PortList {
    usize len;
    char** names;
    WireId* wires;
}
```

### `SimulationErrors`
```
struct SimulationErrors {
    usize conflicts_len;
    WireId* conflicts;
}
```

### `Builder`
An opaque type used to build up a simulation graph.  
Use only behind a pointer.

### `Simulator`
An opaque type representing a simulation.  
Use only behind a pointer.

## Functions

### `string_free`

`Result string_free(char* s)`

Frees a string that was returned by other functions in the API.  
Returns `0` on success.

### `port_list_free`

`Result port_list_free(PortList port_list)`

Frees all allocations of a [`PortList`](#portlist) struct that was returned by other functions in the API.  
Returns `0` on success.

### `simulation_errors_free`

`Result simulation_errors_free(SimulationErrors errors)`

Frees all allocations of a [`SimulationErrors`](#simulationerrors) struct that was returned by other functions in the API.  
Returns `0` on success.

### `logic_state_high_z`

`LogicState* logic_state_high_z()`

Creates a [`LogicState`](#logicstate) with all bits set to the high impedance state.  
The returned `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free).

### `logic_state_undefined`

`LogicState* logic_state_undefined()`

Creates a [`LogicState`](#logicstate) with all bits set to an undefined state.  
The returned `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free).

### `logic_state_logic_0`

`LogicState* logic_state_logic_0()`

Creates a [`LogicState`](#logicstate) with all bits set to the logic low state.  
The returned `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free).

### `logic_state_logic_1`

`LogicState* logic_state_logic_1()`

Creates a [`LogicState`](#logicstate) with all bits set to the logic high state.  
The returned `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free).

### `logic_state_from_int`

`LogicState* logic_state_from_int(u32 value)`

Creates a [`LogicState`](#logicstate) representing the given integer. High bits are set to 0.  
The returned `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free).

### `logic_state_from_big_int`

`Result logic_state_from_big_int(u32* value, usize word_len, LogicState** clone)`

Creates a [`LogicState`](#logicstate) representing the given integer. Integer words are given in little endian order, high bits are set to 0.  
Will fail if `word_len` is not between 1 and 8 inclusive.  
Returns `0` on success.  
The returned `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free).

### `logic_state_parse`

`Result logic_state_parse(char* s, LogicState** state)`

Parses a [`LogicState`](#logicstate) from a string representation. High bits are set to high impedance.  
Will fail if the string is longer than 255 characters, shorter than one character, or contains characters other than `'z'`, `'Z'`, `'x'`, `'X'`, `'0'` and `'1'`.  
Returns `0` on success.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.

### `logic_state_clone`

`Result logic_state_clone(LogicState* state, LogicState** clone)`

Clones a [`LogicState`](#logicstate) into a new allocation.  
Returns `0` on success.  
The cloned `LogicState` must be freed separately by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.

### `logic_state_to_int`

`Result logic_state_to_int(LogicState* state, u8 width, u32* value)`

Attempts to convert the first `width` bits of a [`LogicState`](#logicstate) to an integer.  
`width`  must be between 1 and 32 inclusive. Will fail if any of the bits are either in the `Z` or `X` state.  
Returns `0` on success.

### `logic_state_to_big_int`

`Result logic_state_to_big_int(LogicState* state, u8 width, u32* value)`

Attempts to convert the first `width` bits of a [`LogicState`](#logicstate) to an integer. Integer words are returned in little endian order.
`value` must contain at least `width / 32` words rounded up. Will fail if any of the bits are either in the `Z` or `X` state.
Returns `0` on success.

### `logic_state_get_bit_state`

`Result logic_state_get_bit_state(LogicState* state, u8 bit_index)`

Gets the state of a single bit in a [`LogicState`](#logicstate).  
On success, returns one of the following values:
- `0`: high impedance
- `1`: undefined
- `2`: logic low
- `3`: logic high

### `logic_state_print`

`Result logic_state_print(LogicState* state, u8 width, char* buffer)`

Prints the string representation of a [`LogicState`](#logicstate) into a buffer.  
The buffer must be big enough to hold at least `width` bytes and will not be null terminated by this function.  
Since the buffer is owned by the caller, it must __not__ be freed by [`string_free`](#string_free).  
Returns `0` on success.

### `logic_state_eq`

`Result logic_state_eq(LogicState* a, LogicState* b, u8 width)`

Checks the first `width` bits of two [`LogicState`](#logicstate) objects for equality.  
On success, returns one of the following values:
- `0`: not equal
- `1`: equal

### `logic_state_free`

`Result logic_state_free(LogicState* state)`

Frees a [`LogicState`](#logicstate) that was returned by other functions in the API.  
Returns `0` on success.

### `builder_new`

`Result builder_new(Builder** builder)`

Creates a new [`Builder`](#builder) object.  
The resulting `Builder` must be freed by calling [`simulator_build`](#simulator_build), only if the operation succeeded.  
Returns `0` on success.

### `builder_write_dot`

`Result builder_write_dot(Builder* builder, char* dot_file)`

Writes the simulation graph into a Graphviz DOT file.  
Returns `0` on success.

### `builder_get_wire_width`

`Result builder_get_wire_width(Builder* builder, WireId wire, u8* width)`

Gets the width of a wire.  
Returns `0` on success.

### `builder_set_wire_drive`

`Result builder_set_wire_drive(Builder* builder, WireId wire, LogicState* drive)`

Drives a wire to a certain state without needing a component.  
Returns `0` on success.

### `builder_get_wire_drive`

`Result builder_get_wire_drive(Builder* builder, WireId wire, LogicState** drive)`

Gets the current drive of a wire.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.  
Returns `0` on success.

### `builder_get_register_width`

`Result builder_get_register_width(Builder* builder, ComponentId register, u8* width)`

Gets the width of a register in the simulation.  
The ID passed to `register` must refer to a register component.  
Returns `0` on success.

### `builder_read_register_state`

`Result builder_read_register_state(Builder* builder, ComponentId register, LogicState** state)`

Gets the current state of a register in the simulation.  
The ID passed to `register` must refer to a register component.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.  
Returns `0` on success.

### `builder_write_register_state`

`Result builder_write_register_state(Builder* builder, ComponentId register, LogicState* state)`

Sets the state of a register in the simulation.  
The ID passed to `register` must refer to a register component.  
Returns `0` on success.

### `builder_get_memory_metrics`

`Result builder_get_memory_metrics(Builder* builder, ComponentId memory, usize* size, u8* width)`

Gets the size and width of a memory block in the simulation.  
The ID passed to `memory` must refer to a memory component.  
Returns `0` on success.

### `builder_read_memory_state`

`Result builder_read_memory_state(Builder* builder, ComponentId memory, usize addr, LogicState** state)`

Gets the current state of a memory location in the simulation.  
The ID passed to `memory` must refer to a memory component.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.  
Returns `0` on success.

### `builder_write_memory_state`

`Result builder_write_memory_state(Builder* builder, ComponentId memory, usize addr, LogicState* state)`

Sets the state of a memory location in the simulation.  
The ID passed to `memory` must refer to a memory component.  
Returns `0` on success.

### `builder_set_wire_name`

`Result builder_set_wire_name(Builder* builder, WireId wire, char** name)`

Sets the name of a wire.  
Returns `0` on success.

### `builder_get_wire_name`

`Result builder_get_wire_name(Builder* builder, WireId wire, char** name)`

Gets the name of a wire, if one has been assigned.  
If no name has been assigned to the wire, name will be set to `null`.  
The resulting string (if any) must be freed by calling [`string_free`](#string_free), only if the operation succeeded.  
Returns `0` on success.

### `builder_set_component_name`

`Result builder_set_component_name(Builder* builder, ComponentId component, char* name)`

Sets the name of a component.  
Returns `0` on success.

### `builder_get_component_name`

`Result builder_get_component_name(Builder* builder, ComponentId component, char** name)`

Gets the name of a component, if one has been assigned.  
If no name has been assigned to the component, name will be set to `null`.  
The resulting string (if any) must be freed by calling [`string_free`](#string_free), only if the operation succeeded.  
Returns `0` on success.

### `builder_add_wire`

`Result builder_add_wire(Builder* builder, u8 width, WireId* wire)`

Adds a wire to the simulation.  
Returns `0` on success.

### `builder_add_and_gate`

`Result builder_add_and_gate(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds an AND gate component to the simulation.  
Returns `0` on success.

### `builder_add_or_gate`

`Result builder_add_or_gate(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds an OR gate component to the simulation.  
Returns `0` on success.

### `builder_add_xor_gate`

`Result builder_add_xor_gate(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds an XOR gate component to the simulation.  
Returns `0` on success.

### `builder_add_nand_gate`

`Result builder_add_nand_gate(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds a NAND gate component to the simulation.  
Returns `0` on success.

### `builder_add_nor_gate`

`Result builder_add_nor_gate(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds a NAND gate component to the simulation.  
Returns `0` on success.

### `builder_add_xnor_gate`

`Result builder_add_xnor_gate(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds an XNOR gate component to the simulation.  
Returns `0` on success.

### `builder_add_merge`

`Result builder_add_merge(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds a merge component to the simulation.  
Returns `0` on success.

### `builder_add_priority_decoder`

`Result builder_add_priority_decoder(Builder* builder, WireId* inputs, usize input_len, WireId output, ComponentId* component)`

Adds a priority decoder component to the simulation.  
Returns `0` on success.

### `builder_add_buffer`

`Result builder_add_buffer(Builder* builder, WireId input, WireId enable, WireId output, ComponentId* component)`

Adds a buffer component to the simulation.  
Returns `0` on success.

### `builder_add_add`

`Result builder_add_add(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds an addition component to the simulation.  
Returns `0` on success.

### `builder_add_sub`

`Result builder_add_sub(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a subtraction component to the simulation.  
Returns `0` on success.

### `builder_add_mul`

`Result builder_add_mul(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a multiplication component to the simulation.  
Returns `0` on success.

### `builder_add_left_shift`

`Result builder_add_left_shift(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a left shift component to the simulation.  
Returns `0` on success.

### `builder_add_logical_right_shift`

`Result builder_add_logical_right_shift(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a logical right shift component to the simulation.  
Returns `0` on success.

### `builder_add_arithmetic_right_shift`

`Result builder_add_arithmetic_right_shift(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds an arithmetic right shift component to the simulation.  
Returns `0` on success.

### `builder_add_compare_equal`

`Result builder_add_compare_equal(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_not_equal`

`Result builder_add_compare_not_equal(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_less_than`

`Result builder_add_compare_less_than(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_greater_than`

`Result builder_add_compare_greater_than(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_less_than_or_equal`

`Result builder_add_compare_less_than_or_equal(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_greater_than_or_equal`

`Result builder_add_compare_greater_than_or_equal(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_less_than_signed`

`Result builder_add_compare_less_than_signed(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_greater_than_signed`

`Result builder_add_compare_greater_than_signed(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_less_than_or_equal_signed`

`Result builder_add_compare_less_than_or_equal_signed(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_compare_greater_than_or_equal_signed`

`Result builder_add_compare_greater_than_or_equal_signed(Builder* builder, WireId input_a, WireId input_b, WireId output, ComponentId* component)`

Adds a comparator component to the simulation.  
Returns `0` on success.

### `builder_add_not_gate`

`Result builder_add_not_gate(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a NOT gate component to the simulation.  
Returns `0` on success.

### `builder_add_neg`

`Result builder_add_neg(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a negation component to the simulation.  
Returns `0` on success.

### `builder_add_horizontal_and_gate`

`Result builder_add_horizontal_and_gate(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a horizontal AND gate component to the simulation.  
Returns `0` on success.

### `builder_add_horizontal_or_gate`

`Result builder_add_horizontal_or_gate(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a horizontal OR gate component to the simulation.  
Returns `0` on success.

### `builder_add_horizontal_xor_gate`

`Result builder_add_horizontal_xor_gate(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a horizontal XOR gate component to the simulation.  
Returns `0` on success.

### `builder_add_horizontal_nand_gate`

`Result builder_add_horizontal_nand_gate(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a horizontal NAND gate component to the simulation.  
Returns `0` on success.

### `builder_add_horizontal_nor_gate`

`Result builder_add_horizontal_nor_gate(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a horizontal NOR gate component to the simulation.  
Returns `0` on success.

### `builder_add_horizontal_xnor_gate`

`Result builder_add_horizontal_xnor_gate(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a horizontal XNOR gate component to the simulation.  
Returns `0` on success.

### `builder_add_zero_extend`

`Result builder_add_zero_extend(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a zero extension component to the simulation.  
Returns `0` on success.

### `builder_add_sign_extend`

`Result builder_add_sign_extend(Builder* builder, WireId input, WireId output, ComponentId* component)`

Adds a sign extension component to the simulation.  
Returns `0` on success.

### `builder_add_slice`

`Result builder_add_slice(Builder* builder, WireId input, u8 offset, WireId output, ComponentId* component)`

Adds a slice component to the simulation.  
Returns `0` on success.

### `builder_add_adder`

`Result builder_add_adder(Builder* builder, WireId input_a, WireId input_b, WireId carry_in, WireId output, WireId carry_out, ComponentId* component)`

Adds an adder component to the simulation.  
Returns `0` on success.

### `builder_add_multiplexer`

`Result builder_add_multiplexer(Builder* builder, WireId* inputs, usize input_len, WireId select, WireId output, ComponentId* component)`

Adds a multiplexer component to the simulation.  
Returns `0` on success.

### `builder_add_register`

`Result builder_add_register(Builder* builder, WireId data_in, WireId data_out, WireId enable, WireId clock, u8 clock_polarity, ComponentId* component)`

Adds a register component to the simulation.  
For `clock_polarity`, valid values are `0` (falling edge) and `1` (rising edge). Any other values are invalid.  
Returns `0` on success.

### `builder_add_ram`

`Result builder_add_ram(Builder* builder, WireId write_addr, WireId data_in, WireId read_addr, WireId data_out, WireId write, WireId clock, u8 clock_polarity, ComponentId* component)`

Adds a RAM component to the simulation.  
For `clock_polarity`, valid values are `0` (falling edge) and `1` (rising edge). Any other values are invalid.  
Returns `0` on success.

### `builder_add_rom`

`Result builder_add_rom(Builder* builder, WireId addr, WireId data, ComponentId* component)`

Adds a ROM component to the simulation.  
Returns `0` on success.

### `builder_import_yosys_module`

`Result builder_import_yosys_module(Builder* builder, char* json_file, PortList* inputs, PortList* outputs)`

Imports a module defined by a Yosys netgraph into the circuit.  
On success, `inputs` and `outputs` will contain a list of the imported modules ports.  
The resulting [`PortList`](#portlist) objects must be freed by calling [`port_list_free`](#port_list_free), only if the operation succeeded.  
Returns `0` on success.

Valid netgraphs can be generated with this command:  
`yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -nordff -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>`

### `simulator_build`

`Result simulator_build(Builder** builder, Simulator** simulator)`

Creates a new [`Simulator`](#simulator) object from a [`Builder`](#builder).  
If the operation succeeded, the specified `Builder` will be freed and be set to `null`.  
The resulting `Simulator` must be freed by calling [`simulator_free`](#simulator_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_build_with_trace`

`Result simulator_build_with_trace(Builder** builder, char* trace_file, Simulator** simulator)`

Creates a new [`Simulator`](#simulator) object from a [`Builder`](#builder), with VCD tracing enabled.  
If the operation succeeded, the specified `Builder` will be freed and be set to `null`.  
The `Builder` may be freed even if the operation failed. In this case it will also be set to `null`.  
The resulting `Simulator` must be freed by calling [`simulator_free`](#simulator_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_write_dot`

`Result simulator_write_dot(Simulator* simulator, char* dot_file, u8 show_states)`

Writes the simulation graph into a Graphviz DOT file.  
If `show_states` is non-zero, the exported graph will display the state of wires and registers.  
Returns `0` on success.

### `simulator_get_wire_width`

`Result simulator_get_wire_width(Simulator* simulator, WireId wire, u8* width)`

Gets the width of a wire.  
Returns `0` on success.

### `simulator_set_wire_drive`

`Result simulator_set_wire_drive(Simulator* simulator, WireId wire, LogicState* drive)`

Drives a wire to a certain state without needing a component.  
Returns `0` on success.

### `simulator_get_wire_drive`

`Result simulator_get_wire_drive(Simulator* simulator, WireId wire, LogicState** drive)`

Gets the current drive of a wire.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_get_wire_state`

`Result simulator_get_wire_state(Simulator* simulator, WireId wire, LogicState** state)`

Gets the current state of a wire.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_get_register_width`

`Result simulator_get_register_width(Simulator* simulator, ComponentId register, u8* width)`

Gets the width of a register in the simulation.  
The ID passed to `register` must refer to a register component.  
Returns `0` on success.

### `simulator_read_register_state`

`Result simulator_read_register_state(Simulator* simulator, ComponentId register, LogicState** state)`

Gets the current state of a register in the simulation.  
The ID passed to `register` must refer to a register component.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_write_register_state`

`Result simulator_write_register_state(Simulator* simulator, ComponentId register, LogicState* state)`

Sets the state of a register in the simulation.  
The ID passed to `register` must refer to a register component.  
Returns `0` on success.

### `simulator_get_memory_metrics`

`Result simulator_get_memory_metrics(Simulator* simulator, ComponentId memory, usize* size, u8* width)`

Gets the size and width of a memory block in the simulation.  
The ID passed to `memory` must refer to a memory component.  
Returns `0` on success.

### `simulator_read_memory_state`

`Result simulator_read_memory_state(Simulator* simulator, ComponentId memory, usize addr, LogicState** state)`

Gets the current state of a memory location in the simulation.  
The ID passed to `memory` must refer to a memory component.  
The resulting `LogicState` must be freed by calling [`logic_state_free`](#logic_state_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_write_memory_state`

`Result simulator_write_memory_state(Simulator* simulator, ComponentId memory, usize addr, LogicState* state)`

Sets the state of a memory location in the simulation.  
The ID passed to `memory` must refer to a memory component.  
Returns `0` on success.

### `simulator_get_wire_name`

`Result simulator_get_wire_name(Simulator* simulator, WireId wire, char** name)`

Gets the name of a wire, if one has been assigned.  
If no name has been assigned to the wire, name will be set to `null`.  
The resulting string (if any) must be freed by calling [`string_free`](#string_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_get_component_name`

`Result simulator_get_component_name(Simulator* simulator, ComponentId component, char** name)`

Gets the name of a component, if one has been assigned.  
If no name has been assigned to the component, name will be set to `null`.  
The resulting string (if any) must be freed by calling [`string_free`](#string_free), only if the operation succeeded.  
Returns `0` on success.

### `simulator_reset`

`Result simulator_reset(Simulator* simulator)`

Resets the simulation.  
Returns `0` on success.

### `simulator_run_sim`

`Result simulator_run_sim(Simulator* simulator, u64 max_steps, SimulationErrors* errors)`

Runs the simulation until it settles, but at most for `max_steps` steps.  
On success, returns one of the following values:
- `0`: the simulation settled within `max_steps` steps
- `1`: the simulation did not settle within `max_steps` steps

If a `Conflict` failure is reported, `errors` will contain additional information about which wires had a driver conflict.  
In this case, `errors` must later be freed by calling [`simulation_errors_free`](#simulation_errors_free).

### `simulator_trace`

`Result simulator_trace(Simulator* simulator, u64 time)`

Writes the current state of the simulation into the simulators associated VCD file at the specified time in nanoseconds.  
Calling this function with a [`Simulator`](#simulator) that was not constructed by [`simulator_build_with_trace`](#simulator_build_with_trace) is not illegal, but will have no effect.  
Returns `0` on success.

### `simulator_free`

`Result simulator_free(Simulator* simulator)`

Frees a [`Simulator`](#simulator) object.  
Returns `0` on success.
