pub fn timeline_items_query(event_limit: u8) -> String {
    format!(
        r#"timelineItems(itemTypes: [CROSS_REFERENCED_EVENT, CONNECTED_EVENT, DISCONNECTED_EVENT], first: {}) {{
                      nodes {{
                        __typename
                        ... on CrossReferencedEvent {{
                          createdAt
                          source {{
                            __typename
                            ... on Issue {{
                              number
                              title
                              url
                              state
                              repository {{
                                owner {{
                                  login
                                }}
                                name
                              }}
                            }}
                            ... on PullRequest {{
                              number
                              title
                              url
                              state
                              repository {{
                                owner {{
                                  login
                                }}
                                name
                              }}
                            }}
                          }}
                          willCloseTarget
                        }}
                        ... on ConnectedEvent {{
                          createdAt
                          subject {{
                            __typename
                            ... on Issue {{
                              number
                              title
                              url
                              state
                              repository {{
                                owner {{
                                  login
                                }}
                                name
                              }}
                            }}
                            ... on PullRequest {{
                              number
                              title
                              url
                              state
                              repository {{
                                owner {{
                                  login
                                }}
                                name
                              }}
                            }}
                          }}
                        }}
                        ... on DisconnectedEvent {{
                          createdAt
                          subject {{
                            __typename
                            ... on Issue {{
                              number
                              title
                              url
                              state
                              repository {{
                                owner {{
                                  login
                                }}
                                name
                              }}
                            }}
                            ... on PullRequest {{
                              number
                              title
                              url
                              state
                              repository {{
                                owner {{
                                  login
                                }}
                                name
                              }}
                            }}
                          }}
                        }}
                      }}
                    }}"#,
        event_limit
    )
}
