import unittest
import gsim
from dataclasses import dataclass

@dataclass
class BinaryGateTestData:
    inputA: gsim.LogicState
    inputB: gsim.LogicState
    output: gsim.LogicState

def add_and_gate(builder, inputA, inputB, output):
    return builder.add_and_gate([inputA, inputB], output)

def add_or_gate(builder, inputA, inputB, output):
    return builder.add_or_gate([inputA, inputB], output)
    
def add_xor_gate(builder, inputA, inputB, output):
    return builder.add_xor_gate([inputA, inputB], output)

def add_nand_gate(builder, inputA, inputB, output):
    return builder.add_nand_gate([inputA, inputB], output)

def add_nor_gate(builder, inputA, inputB, output):
    return builder.add_nor_gate([inputA, inputB], output)
    
def add_xnor_gate(builder, inputA, inputB, output):
    return builder.add_xnor_gate([inputA, inputB], output)

class ComponentTests(unittest.TestCase):
    def binary_gate(self, add_gate, width, testData, maxSteps):
        builder = gsim.SimulatorBuilder()
        inputA = builder.add_wire(width)
        inputB = builder.add_wire(width)
        output = builder.add_wire(width)
        gate = add_gate(builder, inputA, inputB, output)
        sim = builder.build()

        for data in testData:
            sim.set_wire_drive(inputA, data.inputA)
            sim.set_wire_drive(inputB, data.inputB)
            sim.run_sim(maxSteps)
            out_state = sim.get_wire_state(output)
            self.assertTrue(out_state.eq(data.output, width))

    def test_and_gate(self):
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
        ]
        self.binary_gate(add_and_gate, 1, testData, 2)
        self.binary_gate(add_and_gate, 32, testData, 2)
        self.binary_gate(add_and_gate, 33, testData, 2)
        self.binary_gate(add_and_gate, 64, testData, 2)

    def test_or_gate(self):
        testData = [
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),
            
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),

            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),

            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.high_z(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.undefined(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),
        ]
        self.binary_gate(add_or_gate, 1, testData, 2)
        self.binary_gate(add_or_gate, 32, testData, 2)
        self.binary_gate(add_or_gate, 33, testData, 2)
        self.binary_gate(add_or_gate, 64, testData, 2)

    def test_xor_gate(self):
        testData = [
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),
            
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),

            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),

            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),
        ]
        self.binary_gate(add_xor_gate, 1, testData, 2)
        self.binary_gate(add_xor_gate, 32, testData, 2)
        self.binary_gate(add_xor_gate, 33, testData, 2)
        self.binary_gate(add_xor_gate, 64, testData, 2)

    def test_nand_gate(self):
        testData = [
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),
            
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),

            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.high_z(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.undefined(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),

            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),
        ]
        self.binary_gate(add_nand_gate, 1, testData, 2)
        self.binary_gate(add_nand_gate, 32, testData, 2)
        self.binary_gate(add_nand_gate, 33, testData, 2)
        self.binary_gate(add_nand_gate, 64, testData, 2)

    def test_nor_gate(self):
        testData = [
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),
            
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),

            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),

            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.high_z(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.undefined(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),
        ]
        self.binary_gate(add_nor_gate, 1, testData, 2)
        self.binary_gate(add_nor_gate, 32, testData, 2)
        self.binary_gate(add_nor_gate, 33, testData, 2)
        self.binary_gate(add_nor_gate, 64, testData, 2)

    def test_xnor_gate(self):
        testData = [
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.high_z(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),
            
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_0(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.undefined(), gsim.LogicState.logic_1(), gsim.LogicState.undefined()),

            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_0(), gsim.LogicState.logic_1()),
            BinaryGateTestData(gsim.LogicState.logic_0(), gsim.LogicState.logic_1(), gsim.LogicState.logic_0()),

            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.high_z(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.undefined(), gsim.LogicState.undefined()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_0(), gsim.LogicState.logic_0()),
            BinaryGateTestData(gsim.LogicState.logic_1(), gsim.LogicState.logic_1(), gsim.LogicState.logic_1()),
        ]
        self.binary_gate(add_xnor_gate, 1, testData, 2)
        self.binary_gate(add_xnor_gate, 32, testData, 2)
        self.binary_gate(add_xnor_gate, 33, testData, 2)
        self.binary_gate(add_xnor_gate, 64, testData, 2)
