import {ApolloClient, ApolloLink, concat, HttpLink, InMemoryCache, split} from '@apollo/client'
import {GraphQLWsLink} from '@apollo/client/link/subscriptions'
import {createClient} from 'graphql-ws'
import {getMainDefinition} from '@apollo/client/utilities'
import {useAuth} from './useAuth'
import {useMemo} from 'react'

export const useAuthenticatedApolloClient = () => {
    const auth = useAuth()

    return useMemo(() => {
        const httpLink = new HttpLink({ uri: '/api/graphql' })

        const wsLink = new GraphQLWsLink(createClient({
            url: `${window.location.origin.replace("http", "ws")}/api/graphql/ws`,
            connectionParams: {
                token: auth.accessToken,
            },
        }))

        const splitLink = split(
            ({ query }) => {
                const definition = getMainDefinition(query);
                return (
                    definition.kind === 'OperationDefinition' &&
                    definition.operation === 'subscription'
                );
            },
            wsLink,
            httpLink,
        )

        const authMiddleware = new ApolloLink((operation, forward) => {
            // add the authorization to the headers
            operation.setContext(({ headers = {} }) => ({
                headers: {
                    ...headers,
                    Authorization: auth.accessToken,
                }
            }))

            return forward(operation);
        })

        return new ApolloClient({
            cache: new InMemoryCache(),
            link: concat(authMiddleware, splitLink)
        })
    }, [auth.accessToken])
}
