resource "kubernetes_manifest" "customresourcedefinition_hydradoomnodes_hydra_doom" {
  manifest = {
    "apiVersion" = "apiextensions.k8s.io/v1"
    "kind"       = "CustomResourceDefinition"
    "metadata" = {
      "name" = "hydradoomnodes.hydra.doom"
    }
    "spec" = {
      "group" = "hydra.doom"
      "names" = {
        "categories" = [
          "hydradoom",
        ]
        "kind"   = "HydraDoomNode"
        "plural" = "hydradoomnodes"
        "shortNames" = [
          "hydradoomnode",
        ]
        "singular" = "hydradoomnode"
      }
      "scope" = "Namespaced"
      "versions" = [
        {
          "additionalPrinterColumns" = [
            {
              "jsonPath" = ".status.state"
              "name"     = "State"
              "type"     = "string"
            },
            {
              "jsonPath" = ".status.transactions"
              "name"     = "Transactions"
              "type"     = "string"
            },
            {
              "jsonPath" = ".status.localUrl"
              "name"     = "Local URI"
              "type"     = "string"
            },
            {
              "jsonPath" = ".status.externalUrl"
              "name"     = "External URI"
              "type"     = "string"
            },
          ]
          "name" = "v1alpha1"
          "schema" = {
            "openAPIV3Schema" = {
              "description" = "Auto-generated derived type for HydraDoomNodeSpec via `CustomResource`"
              "properties" = {
                "spec" = {
                  "properties" = {
                    "asleep" = {
                      "nullable" = true
                      "type"     = "boolean"
                    }
                    "commitInputs" = {
                      "items" = {
                        "type" = "string"
                      }
                      "type" = "array"
                    }
                    "networkId" = {
                      "format"   = "uint8"
                      "minimum"  = 0
                      "nullable" = true
                      "type"     = "integer"
                    }
                    "offline" = {
                      "nullable" = true
                      "type"     = "boolean"
                    }
                    "resources" = {
                      "nullable" = true
                      "properties" = {
                        "limits" = {
                          "properties" = {
                            "cpu" = {
                              "type" = "string"
                            }
                            "memory" = {
                              "type" = "string"
                            }
                          }
                          "required" = [
                            "cpu",
                            "memory",
                          ]
                          "type" = "object"
                        }
                        "requests" = {
                          "properties" = {
                            "cpu" = {
                              "type" = "string"
                            }
                            "memory" = {
                              "type" = "string"
                            }
                          }
                          "required" = [
                            "cpu",
                            "memory",
                          ]
                          "type" = "object"
                        }
                      }
                      "required" = [
                        "limits",
                        "requests",
                      ]
                      "type" = "object"
                    }
                    "seedInput" = {
                      "type" = "string"
                    }
                    "startChainFrom" = {
                      "nullable" = true
                      "type"     = "string"
                    }
                  }
                  "required" = [
                    "commitInputs",
                    "seedInput",
                  ]
                  "type" = "object"
                }
                "status" = {
                  "nullable" = true
                  "properties" = {
                    "externalUrl" = {
                      "type" = "string"
                    }
                    "localUrl" = {
                      "type" = "string"
                    }
                    "state" = {
                      "type" = "string"
                    }
                    "transactions" = {
                      "format" = "int64"
                      "type"   = "integer"
                    }
                  }
                  "required" = [
                    "externalUrl",
                    "localUrl",
                    "state",
                    "transactions",
                  ]
                  "type" = "object"
                }
              }
              "required" = [
                "spec",
              ]
              "title" = "HydraDoomNode"
              "type"  = "object"
            }
          }
          "served"  = true
          "storage" = true
          "subresources" = {
            "status" = {}
          }
        },
      ]
    }
  }
}
