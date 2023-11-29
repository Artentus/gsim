import unittest
import os
import gsim
from dataclasses import dataclass

scriptDir = os.path.dirname(__file__)

@dataclass
class BinaryGateTestData:
    inputA: gsim.LogicState
    inputB: gsim.LogicState
    output: gsim.LogicState

class YosysImportTests(unittest.TestCase):
    def test_simple_and_gate(self):
        testData = [
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),
            
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),

            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.high_z(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.undefined(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),

            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),

            BinaryGateTestData(gsim.LogicState(0xAA), gsim.LogicState(0xAA), gsim.LogicState(0xAA)),
            BinaryGateTestData(gsim.LogicState(0x55), gsim.LogicState(0x55), gsim.LogicState(0x55)),
            BinaryGateTestData(gsim.LogicState(0xAA), gsim.LogicState(0x55), gsim.LogicState(0x00)),
        ]

        builder = gsim.SimulatorBuilder()
        (inputs, outputs) = builder.import_yosys_module(os.path.join(scriptDir, "../import_tests/yosys/simple_and_gate.json"))
        inputA = inputs["a"]
        inputB = inputs["b"]
        output = outputs["o"]
        sim = builder.build()

        for data in testData:
            sim.set_wire_drive(inputA, data.inputA)
            sim.set_wire_drive(inputB, data.inputB)
            sim.run_sim(2)
            out_state = sim.get_wire_state(output)
            self.assertTrue(out_state.eq(data.output, 8))
