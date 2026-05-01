#!/usr/bin/env bash

glslc shaders/gravity.comp -o shaders/gravity.spv
glslc shaders/fade.comp -o shaders/fade.spv
glslc shaders/trail.vert -o shaders/trail.spv
glslc shaders/main.frag -o shaders/frag.spv
glslc shaders/body.vert -o shaders/body.spv
glslc shaders/text.vert -o shaders/text.spv
glslc shaders/texf.frag -o shaders/texf.spv