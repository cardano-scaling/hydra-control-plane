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
              "jsonPath" = ".status.nodeState"
              "name"     = "Node State"
              "type"     = "string"
            },
            {
              "jsonPath" = ".status.gameState"
              "name"     = "Game State"
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
                    "snapshot" = {
                      "nullable" = true
                      "type"     = "string"
                    }
                    "startChainFrom" = {
                      "nullable" = true
                      "type"     = "string"
                    }
                  }
                  "type" = "object"
                }
                "status" = {
                  "nullable" = true
                  "properties" = {
                    "externalUrl" = {
                      "type" = "string"
                    }
                    "gameState" = {
                      "type" = "string"
                    }
                    "localUrl" = {
                      "type" = "string"
                    }
                    "nodeState" = {
                      "type" = "string"
                    }
                    "transactions" = {
                      "format" = "int64"
                      "type"   = "integer"
                    }
                  }
                  "required" = [
                    "externalUrl",
                    "gameState",
                    "localUrl",
                    "nodeState",
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
