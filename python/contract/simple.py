#!/usr/bin/python3
import tensorflow as tf
from keras.layers import Dense
from keras.activations import relu
import argparse

SIMPLE_STATE_SIZE = 221
ACTION_SIZE = 2
INPUT_SIZE = SIMPLE_STATE_SIZE + ACTION_SIZE
INPUT_SHAPE = (1, INPUT_SIZE)
class SimpleModel(tf.keras.Model):
    def __init__(self, *args, **kvargs):
        super(SimpleModel, self).__init__(*args, **kvargs)
        self.dense_in_1 = Dense(64, name="internal_dense_1", input_shape=INPUT_SHAPE)
        self.dense_in_2 = Dense(64, name="dense_internal_2", activation=relu)
        self.last = Dense(1, name="last")

    @tf.function
    def call(self, inputs):
        x = self.dense_in_1(inputs)
        x2 = self.dense_in_2(x)
        #x2 = self.dense_in_2(x)
        return self.last(x2)

    @tf.function
    def training(self, train_data):
        loss = self.train_step(train_data)['loss']
        return loss



def parse_args():
    parser = argparse.ArgumentParser(
        prog='SimpleContractModel',
        description='What the program does',
        epilog='Text at the bottom of help')

    parser.add_argument('output')
    return parser.parse_args()



def main():
    args = parse_args()
    test_model = SimpleModel()
    test_model.compile(optimizer='sgd', loss='mse', metrics=['mae'])

    pred_output = test_model.call.get_concrete_function(tf.TensorSpec(shape=[1, INPUT_SIZE],
                                                                      dtype=tf.uint8,
                                                                      name='inputs'))

    train_output = test_model.training.get_concrete_function((tf.TensorSpec(shape=[1, INPUT_SIZE],
                                          dtype=tf.uint8, name="training_input"),
                            tf.TensorSpec(shape=[1, 1],
                                          dtype=tf.uint8,
                                          name="training_target")))

    write_model = test_model.training.get_concrete_function((tf.TensorSpec(shape=[1, INPUT_SIZE],
                                                                            dtype=tf.uint8, name="training_input"),
                                                              tf.TensorSpec(shape=[1, 1],
                                                                            dtype=tf.uint8,
                                                                            name="training_target")))

    test_model.save(args.output, save_format="tf",
                    signatures = {
                        'train': train_output,
                        'pred': pred_output,
                        #'ckpt_write': ckpt_write,
                    })

if __name__ == "__main__":
    main()